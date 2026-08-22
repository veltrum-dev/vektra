#[allow(dead_code)] // Cargo 将共享 support 分别编译进常规与压力 bench 二进制。
mod support;

use criterion::{
    BatchSize, BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use gpui::{AppContext, SharedString, TestAppContext};
use std::{hint::black_box, sync::OnceLock, time::Duration};
use support::{
    AllocationRecorder, InputFixture, LazySelectFixture, ScrollbarFixture, SelectFixture,
    TooltipFixture, VirtualListFixture, WallFixture, WallKind, component_wall, consume, icon_wall,
    mixed_text, select_tree, tooltip_wall,
};
use vektra::{InputState, ScrollAxis, ScrollGutter};

const MILLION: usize = 1_000_000;
const HUNDRED_THOUSAND: usize = 100_000;
const SIXTEEN_MIB: usize = 16 * 1024 * 1024;
const TEN_MILLION: usize = 10_000_000;

fn select_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress/select");
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(MILLION as u64));
    group.bench_function(BenchmarkId::new("public_tree_build", MILLION), |b| {
        b.iter(|| consume(select_tree(MILLION, 10, MILLION / 2)));
    });

    let recorder = AllocationRecorder::default();
    group.bench_function(
        BenchmarkId::new("allocation_observed_public_tree_build", MILLION),
        |b| {
            b.iter(|| black_box(recorder.measure(|| select_tree(MILLION, 10, MILLION / 2))));
        },
    );
    recorder.report("stress/select/allocation_observed_public_tree_build/1000000");

    group.throughput(Throughput::Elements(HUNDRED_THOUSAND as u64));
    group.bench_function(
        BenchmarkId::new("window_ready_first_draw", HUNDRED_THOUSAND),
        |b| {
            b.iter_batched(
                || SelectFixture::new(HUNDRED_THOUSAND, 10, HUNDRED_THOUSAND / 2, false),
                |mut fixture| fixture.draw(),
                BatchSize::PerIteration,
            );
        },
    );
    group.bench_function(
        BenchmarkId::new("first_open_and_draw", HUNDRED_THOUSAND),
        |b| {
            b.iter_batched(
                || {
                    let mut fixture =
                        SelectFixture::new(HUNDRED_THOUSAND, 10, HUNDRED_THOUSAND / 2, true);
                    fixture.focus_trigger();
                    fixture
                },
                |mut fixture| {
                    fixture.key("down");
                    fixture.draw();
                },
                BatchSize::PerIteration,
            );
        },
    );
    for key in ["end", "pagedown", "b"] {
        group.bench_function(
            BenchmarkId::new(format!("{key}_and_draw"), HUNDRED_THOUSAND),
            |b| {
                b.iter_batched(
                    || {
                        let mut fixture =
                            SelectFixture::new(HUNDRED_THOUSAND, 10, HUNDRED_THOUSAND / 2, true);
                        fixture.open();
                        fixture
                    },
                    |mut fixture| {
                        fixture.key(key);
                        fixture.draw();
                    },
                    BatchSize::PerIteration,
                );
            },
        );
    }
    group.finish();
}

fn input_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress/input");
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Bytes(SIXTEEN_MIB as u64));
    group.bench_function(BenchmarkId::new("initialize", SIXTEEN_MIB), |b| {
        let first = sixteen_mib_text(false);
        b.iter_batched(
            TestAppContext::single,
            |mut cx| consume(cx.new(|cx| InputState::new(first.clone(), cx))),
            BatchSize::PerIteration,
        );
    });
    group.bench_function(BenchmarkId::new("equal_size_set_value", SIXTEEN_MIB), |b| {
        let first = sixteen_mib_text(false);
        let second = sixteen_mib_text(true);
        let mut fixture = InputFixture::new(first.clone(), false);
        let mut alternate = false;
        b.iter(|| {
            alternate = !alternate;
            fixture.set_value(if alternate {
                second.clone()
            } else {
                first.clone()
            });
        });
    });
    group.bench_function(BenchmarkId::new("complete_first_draw", SIXTEEN_MIB), |b| {
        let first = sixteen_mib_text(false);
        b.iter_batched(
            || InputFixture::new(first.clone(), false),
            |mut fixture| fixture.draw(),
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

fn sixteen_mib_text(alternate: bool) -> &'static SharedString {
    static FIRST: OnceLock<SharedString> = OnceLock::new();
    static SECOND: OnceLock<SharedString> = OnceLock::new();
    if alternate {
        SECOND.get_or_init(|| mixed_text(SIXTEEN_MIB, true))
    } else {
        FIRST.get_or_init(|| mixed_text(SIXTEEN_MIB, false))
    }
}

fn wall_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress/component_wall");
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(HUNDRED_THOUSAND as u64));
    group.bench_function(BenchmarkId::new("mixed_build", HUNDRED_THOUSAND), |b| {
        b.iter(|| consume(component_wall(HUNDRED_THOUSAND, WallKind::Mixed, 10)));
    });
    group.bench_function(
        BenchmarkId::new("mixed_complete_first_draw", HUNDRED_THOUSAND),
        |b| {
            b.iter_batched(
                || WallFixture::new(HUNDRED_THOUSAND, WallKind::Mixed, false),
                |mut fixture| fixture.draw(),
                BatchSize::PerIteration,
            );
        },
    );
    group.finish();
}

fn scrollbar_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress/scrollbar");
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(MILLION as u64));
    group.bench_function(BenchmarkId::new("both_stable_first_draw", MILLION), |b| {
        b.iter_batched(
            || ScrollbarFixture::new(MILLION, ScrollAxis::Both, ScrollGutter::Stable, false),
            |mut fixture| fixture.draw(),
            BatchSize::PerIteration,
        );
    });
    group.bench_function(BenchmarkId::new("both_stable_scroll_end", MILLION), |b| {
        b.iter_batched(
            || ScrollbarFixture::new(MILLION, ScrollAxis::Both, ScrollGutter::Stable, true),
            |mut fixture| fixture.scroll_fraction_and_draw(1.0),
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

fn lazy_collection_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress/lazy_collections");
    group.sampling_mode(SamplingMode::Flat);
    for count in [MILLION, TEN_MILLION] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::new("virtual_list_first_draw", count), |b| {
            b.iter_batched(
                || VirtualListFixture::new(count, false),
                |mut fixture| fixture.draw(),
                BatchSize::PerIteration,
            );
        });
        group.bench_function(BenchmarkId::new("virtual_list_steady_redraw", count), |b| {
            let mut fixture = VirtualListFixture::new(count, true);
            b.iter(|| fixture.draw());
        });
        group.bench_function(BenchmarkId::new("virtual_list_jump_90pct", count), |b| {
            b.iter_batched(
                || VirtualListFixture::new(count, true),
                |mut fixture| fixture.jump_and_draw(count * 9 / 10),
                BatchSize::PerIteration,
            );
        });
    }

    group.throughput(Throughput::Elements(MILLION as u64));
    group.bench_function(BenchmarkId::new("select_first_open", MILLION), |b| {
        b.iter_batched(
            || LazySelectFixture::new(MILLION, true),
            |mut fixture| fixture.open(),
            BatchSize::PerIteration,
        );
    });
    group.bench_function(BenchmarkId::new("select_end_and_draw", MILLION), |b| {
        b.iter_batched(
            || {
                let mut fixture = LazySelectFixture::new(MILLION, true);
                fixture.open();
                fixture
            },
            |mut fixture| fixture.end_and_draw(),
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

fn coverage_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress/coverage");
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(10_000));
    group.bench_function(BenchmarkId::new("tooltip_build", 10_000), |b| {
        b.iter(|| consume(tooltip_wall(10_000)));
    });
    group.bench_function(BenchmarkId::new("icon_same_path_build", 100_000), |b| {
        b.iter(|| consume(icon_wall(100_000, false)));
    });
    group.throughput(Throughput::Elements(1_000));
    group.bench_function(BenchmarkId::new("tooltip_focus_delay_draw", 1_000), |b| {
        b.iter_batched(
            || TooltipFixture::new(1_000, true),
            |mut fixture| fixture.focus_delay_and_draw(),
            BatchSize::PerIteration,
        );
    });
    group.finish();
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
    targets = select_stress, input_stress, wall_stress, scrollbar_stress, lazy_collection_stress, coverage_stress
}
criterion_main!(benches);
