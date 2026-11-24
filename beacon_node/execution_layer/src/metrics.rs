pub use metrics::*;
use std::sync::LazyLock;

pub const HIT: &str = "hit";
pub const MISS: &str = "miss";
pub const GET_PAYLOAD: &str = "get_payload";
pub const GET_BLINDED_PAYLOAD: &str = "get_blinded_payload";
pub const GET_BLINDED_PAYLOAD_LOCAL: &str = "get_blinded_payload_local";
pub const GET_BLINDED_PAYLOAD_BUILDER: &str = "get_blinded_payload_builder";
pub const POST_BLINDED_PAYLOAD_BUILDER: &str = "post_blinded_payload_builder";
pub const NEW_PAYLOAD: &str = "new_payload";
pub const FORKCHOICE_UPDATED: &str = "forkchoice_updated";
pub const LOCAL: &str = "local";
pub const BUILDER: &str = "builder";
pub const SUCCESS: &str = "success";
pub const FAILURE: &str = "failure";

pub static EXECUTION_LAYER_PROPOSER_INSERTED: LazyLock<Result<IntCounter>> = LazyLock::new(|| {
    try_create_int_counter(
        "execution_layer_proposer_inserted",
        "Count of times a new proposer is known",
    )
});
pub static EXECUTION_LAYER_PROPOSER_DATA_UPDATED: LazyLock<Result<IntCounter>> =
    LazyLock::new(|| {
        try_create_int_counter(
            "execution_layer_proposer_data_updated",
            "Count of times new proposer data is supplied",
        )
    });
pub static EXECUTION_LAYER_REQUEST_TIMES: LazyLock<Result<HistogramVec>> = LazyLock::new(|| {
    try_create_histogram_vec_with_buckets(
        "execution_layer_request_times",
        "Duration of calls to ELs",
        decimal_buckets(-2, 1),
        &["method"],
    )
});
pub static EXECUTION_LAYER_PAYLOAD_ATTRIBUTES_LOOKAHEAD: LazyLock<Result<Histogram>> =
    LazyLock::new(|| {
        try_create_histogram_with_buckets(
            "execution_layer_payload_attributes_lookahead",
            "Duration between an fcU call with PayloadAttributes and when the block should be produced",
            // The payload is for the next slot, so the lookahead is at most 12s on mainnet.
            Ok(vec![
                0.1, 0.25, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0,
            ]),
        )
    });
pub static EXECUTION_LAYER_PRE_PREPARED_PAYLOAD_ID: LazyLock<Result<IntCounterVec>> = LazyLock::new(
    || {
        try_create_int_counter_vec(
            "execution_layer_pre_prepared_payload_id",
            "Indicates hits or misses for already having prepared a payload id before payload production",
            &["event"],
        )
    },
);
pub static EXECUTION_LAYER_VERIFY_BLOCK_HASH: LazyLock<Result<Histogram>> = LazyLock::new(|| {
    try_create_histogram_with_buckets(
        "execution_layer_verify_block_hash_time",
        "Time to verify the execution block hash in Lighthouse, without the EL",
        Ok(vec![
            10e-6, 50e-6, 100e-6, 500e-6, 1e-3, 5e-3, 10e-3, 50e-3, 100e-3, 500e-3,
        ]),
    )
});
pub static EXECUTION_LAYER_PAYLOAD_STATUS: LazyLock<Result<IntCounterVec>> = LazyLock::new(|| {
    try_create_int_counter_vec(
        "execution_layer_payload_status",
        "Indicates the payload status returned for a particular method",
        &["method", "status"],
    )
});
pub static EXECUTION_LAYER_GET_PAYLOAD_OUTCOME: LazyLock<Result<IntCounterVec>> =
    LazyLock::new(|| {
        try_create_int_counter_vec(
            "execution_layer_get_payload_outcome",
            "The success/failure outcomes from calling get_payload",
            &["outcome"],
        )
    });
pub static EXECUTION_LAYER_BUILDER_REVEAL_PAYLOAD_OUTCOME: LazyLock<Result<IntCounterVec>> =
    LazyLock::new(|| {
        try_create_int_counter_vec(
            "execution_layer_builder_reveal_payload_outcome",
            "The success/failure outcomes from a builder un-blinding a payload",
            &["outcome"],
        )
    });
pub static EXECUTION_LAYER_GET_PAYLOAD_SOURCE: LazyLock<Result<IntCounterVec>> =
    LazyLock::new(|| {
        try_create_int_counter_vec(
            "execution_layer_get_payload_source",
            "The source of each payload returned from get_payload",
            &["source"],
        )
    });
pub static EXECUTION_LAYER_GET_PAYLOAD_BUILDER_REJECTIONS: LazyLock<Result<IntCounterVec>> =
    LazyLock::new(|| {
        try_create_int_counter_vec(
            "execution_layer_get_payload_builder_rejections",
            "The reasons why a payload from a builder was rejected",
            &["reason"],
        )
    });
pub static EXECUTION_LAYER_PAYLOAD_BIDS: LazyLock<Result<IntGaugeVec>> = LazyLock::new(|| {
    try_create_int_gauge_vec(
        "execution_layer_payload_bids",
        "The gwei bid value of payloads received by local EEs or builders. Only shows values up to i64::MAX.",
        &["source"],
    )
});
pub static EXECUTION_LAYER_INFO: LazyLock<Result<IntGaugeVec>> = LazyLock::new(|| {
    try_create_int_gauge_vec(
        "execution_layer_info",
        "The build of the execution layer connected to lighthouse",
        &["code", "name", "version", "commit"],
    )
});

pub fn reset_execution_layer_info_gauge() {
    let _ = EXECUTION_LAYER_INFO.as_ref().map(|gauge| gauge.reset());
}

pub fn expose_execution_layer_info(els: &Vec<crate::ClientVersionV1>) {
    for el in els {
        set_gauge_vec(
            &EXECUTION_LAYER_INFO,
            &[
                &el.code.to_string(),
                &el.name,
                &el.version,
                &el.commit.to_string(),
            ],
            1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histogram_bucket_bounds(name: &str) -> Vec<f64> {
        let family = gather()
            .into_iter()
            .find(|family| family.get_name() == name)
            .unwrap_or_else(|| panic!("{name} should be registered"));
        family.get_metric()[0]
            .get_histogram()
            .get_bucket()
            .iter()
            .map(|bucket| bucket.get_upper_bound())
            .collect()
    }

    #[test]
    fn payload_attributes_lookahead_histogram_covers_a_slot() {
        assert!(EXECUTION_LAYER_PAYLOAD_ATTRIBUTES_LOOKAHEAD.is_ok());

        let name = "execution_layer_payload_attributes_lookahead";
        let bounds = histogram_bucket_bounds(name);
        assert_ne!(bounds, DEFAULT_BUCKETS, "{name} uses the default buckets");
        // The lookahead is at most one slot, 12s on mainnet.
        assert!(
            bounds.last().is_some_and(|top| *top >= 12.0),
            "{name} buckets {bounds:?} stop within one slot"
        );
    }
}
