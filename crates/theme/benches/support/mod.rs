use allocation_counter::AllocationInfo;
use std::{cell::Cell, fmt::Write as _};
use vektra_theme::dtcg::{ResolvedTokens, parse_token_sets};

pub const TOKEN_SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];

pub const DEFAULT_SOURCES: &[&str] = &[
    include_str!("../../../../assets/themes/default/foundation.json"),
    include_str!("../../../../assets/themes/default/light.json"),
    include_str!("../../../../assets/themes/default/button.json"),
    include_str!("../../../../assets/themes/default/input.json"),
    include_str!("../../../../assets/themes/default/checkbox.json"),
    include_str!("../../../../assets/themes/default/radio.json"),
    include_str!("../../../../assets/themes/default/switch.json"),
    include_str!("../../../../assets/themes/default/tooltip.json"),
    include_str!("../../../../assets/themes/default/scrollbar.json"),
    include_str!("../../../../assets/themes/default/select.json"),
];

#[derive(Default)]
pub struct AllocationRecorder {
    samples: Cell<u64>,
    count_total: Cell<u128>,
    count_current: Cell<i128>,
    count_max: Cell<u128>,
    bytes_total: Cell<u128>,
    bytes_current: Cell<i128>,
    bytes_max: Cell<u128>,
}

impl AllocationRecorder {
    pub fn measure<T>(&self, operation: impl FnOnce() -> T) -> T {
        let mut output = None;
        let info = allocation_counter::measure(|| output = Some(operation()));
        self.record(info);
        output.expect("allocation measurement must run its operation")
    }

    pub fn report(&self, benchmark: &str) {
        let samples = self.samples.get();
        if samples == 0 {
            return;
        }
        let divisor = samples as f64;
        eprintln!(
            "VEKTRA_ALLOCATION benchmark={benchmark} samples={samples} allocations/op={:.2} \
             allocated_bytes/op={:.2} net_allocations/op={:.2} net_bytes/op={:.2} \
             peak_allocations/op={:.2} peak_bytes/op={:.2}",
            self.count_total.get() as f64 / divisor,
            self.bytes_total.get() as f64 / divisor,
            self.count_current.get() as f64 / divisor,
            self.bytes_current.get() as f64 / divisor,
            self.count_max.get() as f64 / divisor,
            self.bytes_max.get() as f64 / divisor,
        );
    }

    fn record(&self, info: AllocationInfo) {
        self.samples.set(self.samples.get() + 1);
        self.count_total
            .set(self.count_total.get() + u128::from(info.count_total));
        self.count_current
            .set(self.count_current.get() + i128::from(info.count_current));
        self.count_max
            .set(self.count_max.get() + u128::from(info.count_max));
        self.bytes_total
            .set(self.bytes_total.get() + u128::from(info.bytes_total));
        self.bytes_current
            .set(self.bytes_current.get() + i128::from(info.bytes_current));
        self.bytes_max
            .set(self.bytes_max.get() + u128::from(info.bytes_max));
    }
}

pub fn synthetic_json(count: usize, generation: usize) -> String {
    synthetic_json_with_stride(count, generation, 1)
}

pub fn synthetic_overlay_json(count: usize, generation: usize) -> String {
    synthetic_json_with_stride(count, generation, 10)
}

pub fn parse_profile_with_extra(extra_tokens: usize) -> ResolvedTokens {
    let extra = synthetic_json(extra_tokens, 0);
    let mut sources = Vec::with_capacity(DEFAULT_SOURCES.len() + 1);
    sources.extend_from_slice(DEFAULT_SOURCES);
    sources.push(&extra);
    parse_token_sets(&sources).expect("default profile plus valid benchmark tokens must parse")
}

fn synthetic_json_with_stride(count: usize, generation: usize, stride: usize) -> String {
    let estimated_tokens = count.div_ceil(stride);
    let mut json = String::with_capacity(estimated_tokens.saturating_mul(64));
    json.push_str("{\"benchmark\":{\"$type\":\"number\"");
    for index in (0..count).step_by(stride) {
        write!(
            json,
            ",\"token-{index:07}\":{{\"$value\":{}}}",
            index + generation
        )
        .expect("writing JSON into a String cannot fail");
    }
    json.push_str("}}");
    json
}
