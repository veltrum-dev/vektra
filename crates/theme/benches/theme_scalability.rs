mod support;

use criterion::{BenchmarkId, Criterion, Throughput};
use serde_json::Value;
use std::{hint::black_box, time::Duration};
use support::{
    AllocationRecorder, DEFAULT_SOURCES, TOKEN_SIZES, parse_profile_with_extra, synthetic_json,
    synthetic_overlay_json,
};
use vektra_theme::{
    ResolvedTheme, ResolvedThemeMode, default_theme, default_tokens, dtcg::parse_token_sets,
    profile,
};

fn json_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme/json_parse");
    for &count in TOKEN_SIZES {
        let source = synthetic_json(count, 0);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("synthetic_valid_dtcg", count),
            &count,
            |b, _| {
                b.iter(|| {
                    black_box(
                        serde_json::from_str::<Value>(black_box(&source))
                            .expect("synthetic DTCG JSON must remain valid"),
                    )
                });
            },
        );
    }
    group.finish();
}

fn token_parse_merge_resolve(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme/token_sets");
    for &count in TOKEN_SIZES {
        let base = synthetic_json(count, 0);
        let overlay = synthetic_overlay_json(count, 1);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("single_set_parse_resolve", count),
            &count,
            |b, _| {
                b.iter(|| {
                    black_box(
                        parse_token_sets(&[black_box(base.as_str())])
                            .expect("synthetic token set must parse"),
                    )
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("base_overlay_parse_merge_resolve", count),
            &count,
            |b, _| {
                b.iter(|| {
                    black_box(
                        parse_token_sets(&[black_box(base.as_str()), black_box(overlay.as_str())])
                            .expect("synthetic token sets must merge"),
                    )
                });
            },
        );

        let recorder = AllocationRecorder::default();
        let name = format!("theme/token_sets/allocation_observed_single_set/{count}");
        group.bench_with_input(
            BenchmarkId::new("allocation_observed_single_set", count),
            &count,
            |b, _| {
                b.iter(|| {
                    recorder.measure(|| {
                        black_box(
                            parse_token_sets(&[black_box(base.as_str())])
                                .expect("synthetic token set must parse"),
                        )
                    })
                });
            },
        );
        recorder.report(&name);
    }
    group.finish();
}

fn profile_and_resolved_theme(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme/profile_plus_extra_tokens");
    for &count in TOKEN_SIZES {
        let tokens = parse_profile_with_extra(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("validate", count), &count, |b, _| {
            b.iter(|| {
                profile::validate(black_box(&tokens)).expect("prepared profile must validate")
            });
        });
        group.bench_with_input(
            BenchmarkId::new("resolved_theme_build", count),
            &count,
            |b, _| {
                b.iter(|| {
                    black_box(
                        ResolvedTheme::from_tokens(
                            ResolvedThemeMode::Light,
                            black_box(tokens.clone()),
                        )
                        .expect("prepared profile must build a resolved theme"),
                    )
                });
            },
        );
    }
    group.finish();
}

fn real_default_theme(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme/default_real");
    let real_token_count = parse_token_sets(DEFAULT_SOURCES)
        .expect("built-in sources must parse")
        .len();
    group.throughput(Throughput::Elements(real_token_count as u64));
    for (name, mode) in [
        ("light", ResolvedThemeMode::Light),
        ("dark", ResolvedThemeMode::Dark),
    ] {
        group.bench_function(format!("cold_load_parse_validate/{name}"), |b| {
            b.iter(|| {
                black_box(default_tokens(mode).expect("built-in default theme must remain valid"))
            });
        });
        group.bench_function(format!("cold_load_parse_validate_resolve/{name}"), |b| {
            b.iter(|| {
                let tokens =
                    default_tokens(mode).expect("built-in default theme must remain valid");
                black_box(
                    ResolvedTheme::from_tokens(mode, tokens)
                        .expect("built-in default theme must resolve"),
                )
            });
        });

        let _ = default_theme(mode);
        group.bench_function(format!("cached_arc_read/{name}"), |b| {
            b.iter(|| black_box(default_theme(mode)));
        });
    }
    group.finish();
}

fn regular_criterion() -> Criterion {
    Criterion::default()
        .without_plots()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_millis(750))
        .configure_from_args()
}

fn smoke(c: &mut Criterion) {
    c.bench_function("harness/default_theme_parse_validate", |b| {
        b.iter(|| {
            black_box(
                default_tokens(ResolvedThemeMode::Light)
                    .expect("built-in default theme must remain valid"),
            )
        });
    });
}

fn main() {
    let arguments = std::env::args().collect::<Vec<_>>();
    let smoke_only = !arguments.iter().any(|argument| argument == "--bench")
        || arguments.iter().any(|argument| argument == "--test");
    let mut criterion = regular_criterion();
    if smoke_only {
        smoke(&mut criterion);
    } else {
        json_parse(&mut criterion);
        token_parse_merge_resolve(&mut criterion);
        profile_and_resolved_theme(&mut criterion);
        real_default_theme(&mut criterion);
    }
    criterion.final_summary();
}
