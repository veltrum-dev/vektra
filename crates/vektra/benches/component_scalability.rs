#[allow(dead_code)] // 共享 support 还包含只由 stress target 使用的百万项惰性夹具。
mod support;

use criterion::{BatchSize, BenchmarkId, Criterion, SamplingMode, Throughput};
use gpui::{AppContext, TestAppContext};
use std::{hint::black_box, time::Duration};
use support::{
    AllocationRecorder, INPUT_BYTE_SIZES, InputFixture, InputStateFixture, SCROLLBAR_BUILD_SIZES,
    SCROLLBAR_RENDER_SIZES, SELECT_DATA_SIZES, SELECT_RENDER_SIZES, ScrollbarFixture,
    SelectFixture, TooltipFixture, VIRTUAL_LIST_SIZES, VirtualListFixture, WALL_BUILD_SIZES,
    WALL_RENDER_SIZES, WallFixture, WallKind, component_wall, consume, icon_wall, mixed_text,
    scrollbar_tree, select_tree, tooltip_wall,
};
use vektra::{InputState, ScrollAxis, ScrollGutter};

fn select_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("select/build");
    for &count in SELECT_DATA_SIZES {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("public_tree", count),
            &count,
            |b, &count| {
                b.iter(|| consume(select_tree(black_box(count), 0, 0)));
            },
        );

        let recorder = AllocationRecorder::default();
        let name = format!("select/build/allocation_observed/{count}");
        group.bench_with_input(
            BenchmarkId::new("allocation_observed", count),
            &count,
            |b, &count| {
                b.iter(|| black_box(recorder.measure(|| select_tree(black_box(count), 0, 0))));
            },
        );
        recorder.report(&name);
    }
    group.finish();
}

fn select_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("select/render");
    group.sampling_mode(SamplingMode::Flat);
    for &count in SELECT_RENDER_SIZES {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("window_ready_first_draw", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || SelectFixture::new(count, 0, count / 2, false),
                    |mut fixture| fixture.draw(),
                    BatchSize::PerIteration,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("steady_redraw", count),
            &count,
            |b, &count| {
                let mut fixture = SelectFixture::new(count, 0, count / 2, true);
                b.iter(|| fixture.draw());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("equal_size_update_and_draw", count),
            &count,
            |b, &count| {
                let mut fixture = SelectFixture::new(count, 0, count / 2, true);
                b.iter(|| fixture.update_options());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("first_open_and_draw", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut fixture = SelectFixture::new(count, 0, count / 2, true);
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
        group.bench_with_input(
            BenchmarkId::new("open_steady_redraw", count),
            &count,
            |b, &count| {
                let mut fixture = SelectFixture::new(count, 0, count / 2, true);
                fixture.open();
                b.iter(|| fixture.draw());
            },
        );
    }
    group.finish();
}

fn select_interaction(c: &mut Criterion) {
    let mut group = c.benchmark_group("select/interaction_and_draw");
    group.sampling_mode(SamplingMode::Flat);
    for &count in SELECT_RENDER_SIZES {
        group.throughput(Throughput::Elements(count as u64));
        for key in ["down", "home", "end", "pageup", "pagedown"] {
            group.bench_with_input(BenchmarkId::new(key, count), &count, |b, &count| {
                b.iter_batched(
                    || {
                        let mut fixture = SelectFixture::new(count, 0, count / 2, true);
                        fixture.open();
                        fixture
                    },
                    |mut fixture| {
                        fixture.key(key);
                        fixture.draw();
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }

    let count = 10_000;
    for disabled_percent in [0, 10, 50, 90] {
        for (position, active) in [("start", 0), ("middle", count / 2), ("end", count - 1)] {
            let case = format!("disabled_{disabled_percent}/active_{position}");
            group.bench_function(BenchmarkId::new("typeahead", &case), |b| {
                b.iter_batched(
                    || {
                        let mut fixture = SelectFixture::new(count, disabled_percent, active, true);
                        fixture.open();
                        fixture
                    },
                    |mut fixture| {
                        fixture.key("b");
                        fixture.draw();
                    },
                    BatchSize::PerIteration,
                );
            });
            group.bench_function(BenchmarkId::new("arrow_down", &case), |b| {
                b.iter_batched(
                    || {
                        let mut fixture = SelectFixture::new(count, disabled_percent, active, true);
                        fixture.open();
                        fixture
                    },
                    |mut fixture| {
                        fixture.key("down");
                        fixture.draw();
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }
    group.bench_function("typeahead_after_deterministic_timeout/10000", |b| {
        b.iter_batched(
            || {
                let mut fixture = SelectFixture::new(count, 0, count / 2, true);
                fixture.open();
                fixture.key("b");
                fixture.advance_typeahead_timeout();
                fixture
            },
            |mut fixture| {
                fixture.key("b");
                fixture.draw();
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

fn input_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("input/state");
    for &bytes in INPUT_BYTE_SIZES {
        let first = mixed_text(bytes, false);
        let second = mixed_text(bytes, true);
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("initialize", bytes), &bytes, |b, _| {
            b.iter_batched(
                TestAppContext::single,
                |mut cx| consume(cx.new(|cx| InputState::new(first.clone(), cx))),
                BatchSize::PerIteration,
            );
        });
        group.bench_with_input(
            BenchmarkId::new("equal_size_set_value", bytes),
            &bytes,
            |b, _| {
                let mut fixture = InputStateFixture::new(first.clone());
                let mut alternate = false;
                b.iter(|| {
                    alternate = !alternate;
                    fixture.set_value(if alternate {
                        second.clone()
                    } else {
                        first.clone()
                    });
                    black_box(fixture.value_len());
                });
            },
        );

        let recorder = AllocationRecorder::default();
        let name = format!("input/state/allocation_observed_equal_size_set_value/{bytes}");
        group.bench_with_input(
            BenchmarkId::new("allocation_observed_equal_size_set_value", bytes),
            &bytes,
            |b, _| {
                let mut fixture = InputStateFixture::new(first.clone());
                let mut alternate = false;
                b.iter(|| {
                    alternate = !alternate;
                    recorder.measure(|| {
                        fixture.set_value(if alternate {
                            second.clone()
                        } else {
                            first.clone()
                        });
                    });
                });
            },
        );
        recorder.report(&name);
    }
    group.finish();
}

fn input_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("input/render");
    group.sampling_mode(SamplingMode::Flat);
    for &bytes in INPUT_BYTE_SIZES {
        let text = mixed_text(bytes, false);
        let alternate = mixed_text(bytes, true);
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("cold_window_create_and_initial_draw", bytes),
            &bytes,
            |b, _| b.iter(|| consume(InputFixture::new(text.clone(), false))),
        );
        group.bench_with_input(
            BenchmarkId::new("window_ready_first_draw_accesskit", bytes),
            &bytes,
            |b, _| {
                b.iter_batched(
                    || InputFixture::new(text.clone(), false),
                    |mut fixture| fixture.draw(),
                    BatchSize::PerIteration,
                );
            },
        );
        group.bench_with_input(BenchmarkId::new("steady_redraw", bytes), &bytes, |b, _| {
            let mut fixture = InputFixture::new(text.clone(), true);
            b.iter(|| fixture.draw());
        });
        group.bench_with_input(
            BenchmarkId::new("equal_size_update_and_draw", bytes),
            &bytes,
            |b, _| {
                let mut fixture = InputFixture::new(text.clone(), true);
                let mut toggle = false;
                b.iter(|| {
                    toggle = !toggle;
                    fixture.set_value(if toggle {
                        alternate.clone()
                    } else {
                        text.clone()
                    });
                    fixture.draw();
                });
            },
        );

        let recorder = AllocationRecorder::default();
        let name = format!("input/render/allocation_observed_equal_size_update_and_draw/{bytes}");
        group.bench_with_input(
            BenchmarkId::new("allocation_observed_equal_size_update_and_draw", bytes),
            &bytes,
            |b, _| {
                let mut fixture = InputFixture::new(text.clone(), true);
                let mut toggle = false;
                b.iter(|| {
                    toggle = !toggle;
                    recorder.measure(|| {
                        fixture.set_value(if toggle {
                            alternate.clone()
                        } else {
                            text.clone()
                        });
                    });
                });
            },
        );
        recorder.report(&name);
    }
    group.finish();
}

fn input_interaction(c: &mut Criterion) {
    let mut group = c.benchmark_group("input/interaction_and_draw");
    group.sampling_mode(SamplingMode::Flat);
    for &bytes in INPUT_BYTE_SIZES {
        let text = mixed_text(bytes, false);
        group.throughput(Throughput::Bytes(bytes as u64));
        for (name, key) in [
            ("grapheme_move", "left"),
            ("word_move", "alt-left"),
            ("word_delete", "alt-backspace"),
            ("selection_update", "shift-left"),
        ] {
            group.bench_with_input(BenchmarkId::new(name, bytes), &bytes, |b, _| {
                b.iter_batched(
                    || {
                        let mut fixture = InputFixture::new(text.clone(), true);
                        fixture.focus();
                        fixture
                    },
                    |mut fixture| fixture.key_and_draw(key),
                    BatchSize::PerIteration,
                );
            });
        }
        group.bench_with_input(
            BenchmarkId::new("representative_input", bytes),
            &bytes,
            |b, _| {
                b.iter_batched(
                    || {
                        let mut fixture = InputFixture::new(text.clone(), true);
                        fixture.focus();
                        fixture
                    },
                    |mut fixture| fixture.input_and_draw("A中👩🏽‍💻e\u{301}"),
                    BatchSize::PerIteration,
                );
            },
        );
    }
    group.finish();
}

fn component_wall_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_wall/build");
    for kind in WallKind::ALL {
        for &count in WALL_BUILD_SIZES {
            group.throughput(Throughput::Elements(count as u64));
            group.bench_with_input(BenchmarkId::new(kind.name(), count), &count, |b, &count| {
                b.iter(|| consume(component_wall(count, kind, 0)))
            });
            let recorder = AllocationRecorder::default();
            let name = format!(
                "component_wall/build/allocation_observed/{}/{count}",
                kind.name()
            );
            group.bench_with_input(
                BenchmarkId::new(format!("{}_allocation_observed", kind.name()), count),
                &count,
                |b, &count| {
                    b.iter(|| black_box(recorder.measure(|| component_wall(count, kind, 0))));
                },
            );
            recorder.report(&name);
        }
    }
    group.finish();
}

fn component_wall_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_wall/render");
    group.sampling_mode(SamplingMode::Flat);
    for kind in WallKind::ALL {
        for &count in WALL_RENDER_SIZES {
            group.throughput(Throughput::Elements(count as u64));
            let prefix = kind.name();
            group.bench_with_input(
                BenchmarkId::new(format!("{prefix}_window_ready_first_draw"), count),
                &count,
                |b, &count| {
                    b.iter_batched(
                        || WallFixture::new(count, kind, false),
                        |mut fixture| fixture.draw(),
                        BatchSize::PerIteration,
                    );
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("{prefix}_steady_redraw"), count),
                &count,
                |b, &count| {
                    let mut fixture = WallFixture::new(count, kind, true);
                    b.iter(|| fixture.draw());
                },
            );
            for changed in [10, 100] {
                group.bench_with_input(
                    BenchmarkId::new(format!("{prefix}_{changed}pct_update_and_draw"), count),
                    &count,
                    |b, &count| {
                        b.iter_batched(
                            || WallFixture::new(count, kind, true),
                            |mut fixture| fixture.update_and_draw(changed),
                            BatchSize::PerIteration,
                        );
                    },
                );
            }
        }
    }
    group.finish();
}

fn scrollbar(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrollbar");
    group.sampling_mode(SamplingMode::Flat);
    for axis in [
        ScrollAxis::Vertical,
        ScrollAxis::Horizontal,
        ScrollAxis::Both,
    ] {
        for gutter in [ScrollGutter::Overlay, ScrollGutter::Stable] {
            let mode = format!("{axis:?}/{gutter:?}").to_lowercase();
            for &count in SCROLLBAR_BUILD_SIZES {
                group.throughput(Throughput::Elements(count as u64));
                group.bench_with_input(
                    BenchmarkId::new(format!("build/{mode}"), count),
                    &count,
                    |b, &count| b.iter(|| consume(scrollbar_tree(count, axis, gutter))),
                );
            }
            for &count in SCROLLBAR_RENDER_SIZES {
                group.throughput(Throughput::Elements(count as u64));
                group.bench_with_input(
                    BenchmarkId::new(format!("window_ready_first_layout_draw/{mode}"), count),
                    &count,
                    |b, &count| {
                        b.iter_batched(
                            || ScrollbarFixture::new(count, axis, gutter, false),
                            |mut fixture| fixture.draw(),
                            BatchSize::PerIteration,
                        );
                    },
                );
                group.bench_with_input(
                    BenchmarkId::new(format!("steady_layout_draw/{mode}"), count),
                    &count,
                    |b, &count| {
                        let mut fixture = ScrollbarFixture::new(count, axis, gutter, true);
                        b.iter(|| fixture.draw());
                    },
                );
                for (position, fraction) in [("middle", 0.5), ("end", 1.0)] {
                    group.bench_with_input(
                        BenchmarkId::new(
                            format!("scroll_{position}_thumb_update_draw/{mode}"),
                            count,
                        ),
                        &count,
                        |b, &count| {
                            b.iter_batched(
                                || ScrollbarFixture::new(count, axis, gutter, true),
                                |mut fixture| fixture.scroll_fraction_and_draw(fraction),
                                BatchSize::PerIteration,
                            );
                        },
                    );
                }
            }
        }
    }
    group.finish();
}

fn virtual_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("virtual_list/render");
    group.sampling_mode(SamplingMode::Flat);
    for &count in VIRTUAL_LIST_SIZES {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("first_draw", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || VirtualListFixture::new(count, false),
                    |mut fixture| fixture.draw(),
                    BatchSize::PerIteration,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("steady_redraw", count),
            &count,
            |b, &count| {
                let mut fixture = VirtualListFixture::new(count, true);
                b.iter(|| fixture.draw());
            },
        );
        group.bench_with_input(
            BenchmarkId::new("jump_to_end_and_draw", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || VirtualListFixture::new(count, true),
                    |mut fixture| fixture.jump_and_draw(count.saturating_sub(1)),
                    BatchSize::PerIteration,
                );
            },
        );
    }
    group.finish();
}

fn tooltip_icon_focus(c: &mut Criterion) {
    let mut group = c.benchmark_group("coverage/tooltip_icon_focus");
    group.sampling_mode(SamplingMode::Flat);
    for count in [100usize, 1_000] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::new("tooltip_build", count), |b| {
            b.iter(|| consume(tooltip_wall(count)));
        });
        group.bench_function(BenchmarkId::new("tooltip_first_draw", count), |b| {
            b.iter_batched(
                || TooltipFixture::new(count, false),
                |mut fixture| fixture.draw(),
                BatchSize::PerIteration,
            );
        });
        group.bench_function(BenchmarkId::new("tooltip_focus_delay_draw", count), |b| {
            b.iter_batched(
                || TooltipFixture::new(count, true),
                |mut fixture| fixture.focus_delay_and_draw(),
                BatchSize::PerIteration,
            );
        });
        group.bench_function(BenchmarkId::new("icon_same_path_build", count), |b| {
            b.iter(|| consume(icon_wall(count, false)));
        });
        group.bench_function(BenchmarkId::new("icon_unique_path_build", count), |b| {
            b.iter(|| consume(icon_wall(count, true)));
        });
        group.bench_function(
            BenchmarkId::new("focus_keyed_button_steady_draw", count),
            |b| {
                let mut fixture = WallFixture::new(count, WallKind::Button, true);
                b.iter(|| fixture.draw());
            },
        );
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
    c.bench_function("harness/gpui_test_context_complete_draw", |b| {
        b.iter_batched(
            || SelectFixture::new(10, 0, 5, false),
            |mut fixture| fixture.draw(),
            BatchSize::PerIteration,
        );
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
        select_build(&mut criterion);
        select_render(&mut criterion);
        select_interaction(&mut criterion);
        input_state(&mut criterion);
        input_render(&mut criterion);
        input_interaction(&mut criterion);
        component_wall_build(&mut criterion);
        component_wall_render(&mut criterion);
        scrollbar(&mut criterion);
        virtual_list(&mut criterion);
        tooltip_icon_focus(&mut criterion);
    }
    criterion.final_summary();
}
