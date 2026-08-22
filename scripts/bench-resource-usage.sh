#!/usr/bin/env bash

set -euo pipefail

if (( $# < 3 || $# > 4 )); then
    echo "用法: $0 <package> <bench-target> <filter> [features]" >&2
    exit 2
fi

package=$1
target=$2
filter=$3
features=${4:-}

cargo_args=(
    bench
    --package "$package"
    --bench "$target"
    --no-run
    --message-format json-render-diagnostics
)
if [[ -n "$features" ]]; then
    cargo_args+=(--features "$features")
fi

executable=$(
    cargo "${cargo_args[@]}" |
        python3 -c '
import json
import sys

target_name = sys.argv[1]
executables = []
for line in sys.stdin:
    try:
        message = json.loads(line)
    except json.JSONDecodeError:
        continue
    target = message.get("target", {})
    if (
        message.get("reason") == "compiler-artifact"
        and target.get("name") == target_name
        and "bench" in target.get("kind", [])
        and message.get("executable")
    ):
        executables.append(message["executable"])

if len(executables) != 1:
    raise SystemExit(
        f"expected one executable for bench target {target_name!r}, got {len(executables)}"
    )
print(executables[0])
' "$target"
)

case "$(uname -s)" in
    Darwin)
        platform=macos
        time_args=(-l)
        ;;
    Linux)
        platform=linux
        time_args=(-v)
        ;;
    *)
        echo "不支持的平台；Windows 请使用 scripts/bench-resource-usage.ps1" >&2
        exit 2
        ;;
esac

metrics_file=$(mktemp)
trap 'rm -f "$metrics_file"' EXIT

set +e
/usr/bin/time "${time_args[@]}" -o "$metrics_file" \
    "$executable" --bench "$filter" --exact --quick --noplot
status=$?
set -e

cat "$metrics_file" >&2
python3 -c '
import json
import re
import sys

platform, path, exit_status, package, target, benchmark_filter = sys.argv[1:]
text = open(path, encoding="utf-8").read()
if platform == "macos":
    times = re.search(
        r"([0-9.]+) real\s+([0-9.]+) user\s+([0-9.]+) sys", text
    )
    rss = re.search(r"(\d+)\s+maximum resident set size", text)
    if not times or not rss:
        raise SystemExit("unable to parse macOS /usr/bin/time output")
    wall, user, system = map(float, times.groups())
    maximum_rss_bytes = int(rss.group(1))
    cpu_percent = 100.0 * (user + system) / wall if wall else 0.0
else:
    user = re.search(r"User time \(seconds\):\s*([0-9.]+)", text)
    system = re.search(r"System time \(seconds\):\s*([0-9.]+)", text)
    elapsed = re.search(r"Elapsed \(wall clock\) time.*:\s*([0-9:.]+)", text)
    cpu = re.search(r"Percent of CPU this job got:\s*([0-9.]+)%", text)
    rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", text)
    if not all((user, system, elapsed, cpu, rss)):
        raise SystemExit("unable to parse GNU /usr/bin/time output")
    parts = [float(part) for part in elapsed.group(1).split(":")]
    wall = sum(part * (60 ** index) for index, part in enumerate(reversed(parts)))
    user = float(user.group(1))
    system = float(system.group(1))
    cpu_percent = float(cpu.group(1))
    maximum_rss_bytes = int(rss.group(1)) * 1024

total_cpu_seconds = user + system
payload = {
    "schema_version": 1,
    "platform": platform,
    "package": package,
    "bench_target": target,
    "benchmark_filter": benchmark_filter,
    "wall_seconds": round(wall, 6),
    "user_cpu_seconds": round(user, 6),
    "system_cpu_seconds": round(system, 6),
    "total_cpu_seconds": round(total_cpu_seconds, 6),
    "cpu_percent": round(cpu_percent, 2),
    "peak_memory_bytes": maximum_rss_bytes,
    "peak_memory_kind": "maximum_rss",
    "exit_status": int(exit_status),
}

print(
    "VEKTRA_PROCESS_METRICS "
    f"platform={platform} wall_seconds={wall:.6f} user_seconds={user:.6f} "
    f"system_seconds={system:.6f} cpu_percent={cpu_percent:.2f} "
    f"maximum_rss_bytes={maximum_rss_bytes} exit_status={exit_status}"
)
print("VEKTRA_PROCESS_METRICS_JSON " + json.dumps(payload, separators=(",", ":")))
' "$platform" "$metrics_file" "$status" "$package" "$target" "$filter"

exit "$status"
