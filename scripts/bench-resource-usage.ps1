[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Package,

    [Parameter(Mandatory = $true)]
    [string]$Target,

    [Parameter(Mandatory = $true)]
    [string]$Filter,

    [string]$Features = ""
)

$ErrorActionPreference = "Stop"

$cargoArgs = @(
    "bench",
    "--package", $Package,
    "--bench", $Target,
    "--no-run",
    "--message-format", "json-render-diagnostics"
)
if ($Features) {
    $cargoArgs += @("--features", $Features)
}

$artifacts = & cargo @cargoArgs | ForEach-Object {
    try {
        $_ | ConvertFrom-Json
    }
    catch {
        $null
    }
} | Where-Object {
    $_.reason -eq "compiler-artifact" -and
    $_.target.name -eq $Target -and
    $_.target.kind -contains "bench" -and
    $_.executable
}

if ($LASTEXITCODE -ne 0) {
    throw "cargo bench --no-run failed with exit code $LASTEXITCODE"
}
if (@($artifacts).Count -ne 1) {
    throw "Expected one executable for bench target '$Target', got $(@($artifacts).Count)"
}

$startInfo = [System.Diagnostics.ProcessStartInfo]::new()
$startInfo.FileName = $artifacts[0].executable
$startInfo.UseShellExecute = $false
@("--bench", $Filter, "--exact", "--quick", "--noplot") | ForEach-Object {
    [void]$startInfo.ArgumentList.Add($_)
}

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$process = [System.Diagnostics.Process]::Start($startInfo)
$process.WaitForExit()
$stopwatch.Stop()

$exitCode = $process.ExitCode
$wallSeconds = $stopwatch.Elapsed.TotalSeconds
$cpuSeconds = $process.TotalProcessorTime.TotalSeconds
$cpuPercent = if ($wallSeconds -gt 0) { 100.0 * $cpuSeconds / $wallSeconds } else { 0.0 }
$peakWorkingSetBytes = $process.PeakWorkingSet64
$process.Dispose()

$culture = [System.Globalization.CultureInfo]::InvariantCulture
$wall = $wallSeconds.ToString("F6", $culture)
$cpu = $cpuSeconds.ToString("F6", $culture)
$percent = $cpuPercent.ToString("F2", $culture)
Write-Output "VEKTRA_PROCESS_METRICS platform=windows wall_seconds=$wall cpu_seconds=$cpu cpu_percent=$percent peak_working_set_bytes=$peakWorkingSetBytes exit_status=$exitCode"
$payload = [ordered]@{
    schema_version = 1
    platform = "windows"
    package = $Package
    bench_target = $Target
    benchmark_filter = $Filter
    wall_seconds = [double]$wallSeconds
    user_cpu_seconds = $null
    system_cpu_seconds = $null
    total_cpu_seconds = [double]$cpuSeconds
    cpu_percent = [double]$cpuPercent
    peak_memory_bytes = [long]$peakWorkingSetBytes
    peak_memory_kind = "peak_working_set"
    exit_status = [int]$exitCode
}
Write-Output "VEKTRA_PROCESS_METRICS_JSON $($payload | ConvertTo-Json -Compress)"

exit $exitCode
