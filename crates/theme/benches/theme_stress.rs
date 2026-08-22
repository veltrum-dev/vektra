#[allow(dead_code)] // Cargo 将共享 support 分别编译进常规与压力 bench 二进制。
mod support;

use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use serde_json::Value;
use std::{hint::black_box, sync::OnceLock, time::Duration};
use support::{
    AllocationRecorder, parse_profile_with_extra, synthetic_json, synthetic_overlay_json,
};
use vektra_theme::{
    ResolvedTheme, ResolvedThemeMode,
    dtcg::{ResolvedTokens, parse_token_sets},
    profile,
};

const MILLION: usize = 1_000_000;

fn theme_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress/theme");
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(MILLION as u64));
    group.bench_function(BenchmarkId::new("json_parse", MILLION), |b| {
        let source = million_source();
        b.iter(|| {
            black_box(
                serde_json::from_str::<Value>(black_box(source))
                    .expect("million-token JSON must parse"),
            )
        });
    });
    group.bench_function(BenchmarkId::new("single_set_parse_resolve", MILLION), |b| {
        let source = million_source();
        b.iter(|| {
            black_box(
                parse_token_sets(&[black_box(source)])
                    .expect("million-token set must parse and resolve"),
            )
        });
    });
    group.bench_function(
        BenchmarkId::new("base_overlay_parse_merge_resolve", MILLION),
        |b| {
            let source = million_source();
            let overlay = million_overlay();
            b.iter(|| {
                black_box(
                    parse_token_sets(&[black_box(source), black_box(overlay)])
                        .expect("million-token sets must merge and resolve"),
                )
            });
        },
    );
    group.bench_function(BenchmarkId::new("profile_validate", MILLION), |b| {
        let tokens = million_profile();
        b.iter(|| {
            profile::validate(black_box(tokens))
                .expect("default profile plus million valid tokens must validate")
        });
    });
    group.bench_function(BenchmarkId::new("resolved_theme_build", MILLION), |b| {
        let tokens = million_profile();
        b.iter(|| {
            black_box(
                ResolvedTheme::from_tokens(ResolvedThemeMode::Light, black_box(tokens.clone()))
                    .expect("million-token profile must resolve"),
            )
        });
    });

    let recorder = AllocationRecorder::default();
    group.bench_function(
        BenchmarkId::new("allocation_observed_single_set_parse", MILLION),
        |b| {
            let source = million_source();
            b.iter(|| {
                recorder.measure(|| {
                    black_box(
                        parse_token_sets(&[black_box(source)])
                            .expect("million-token set must parse"),
                    )
                })
            });
        },
    );
    recorder.report("stress/theme/allocation_observed_single_set_parse/1000000");
    group.finish();
}

fn million_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| synthetic_json(MILLION, 0))
}

fn million_overlay() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| synthetic_overlay_json(MILLION, 1))
}

fn million_profile() -> &'static ResolvedTokens {
    static TOKENS: OnceLock<ResolvedTokens> = OnceLock::new();
    TOKENS.get_or_init(|| parse_profile_with_extra(MILLION))
}

fn stress_criterion() -> Criterion {
    Criterion::default()
        .without_plots()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_secs(1))
        .configure_from_args()
}

criterion_group! {
    name = benches;
    config = stress_criterion();
    targets = theme_stress
}
criterion_main!(benches);
