pub use metrics::*;
use std::sync::LazyLock;

pub static HTTP_API_PATHS_TOTAL: LazyLock<Result<IntCounterVec>> = LazyLock::new(|| {
    try_create_int_counter_vec(
        "http_api_paths_total",
        "Count of HTTP requests received",
        &["path"],
    )
});
pub static HTTP_API_STATUS_CODES_TOTAL: LazyLock<Result<IntCounterVec>> = LazyLock::new(|| {
    try_create_int_counter_vec(
        "http_api_status_codes_total",
        "Count of HTTP status codes returned",
        &["status"],
    )
});
pub static HTTP_API_PATHS_TIMES: LazyLock<Result<HistogramVec>> = LazyLock::new(|| {
    try_create_histogram_vec(
        "http_api_paths_times",
        "Duration to process HTTP requests per path",
        &["path"],
    )
});

pub static HTTP_API_BLOCK_BROADCAST_DELAY_TIMES: LazyLock<Result<HistogramVec>> =
    LazyLock::new(|| {
        try_create_histogram_vec_with_buckets(
            "http_api_block_broadcast_delay_times",
            "Time between start of the slot and when the block completed broadcast and processing",
            // Resolves the 4s attestation deadline and covers blocks up to two mainnet slots late.
            Ok(vec![
                0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 24.0,
            ]),
            &["provenance"],
        )
    });
pub static HTTP_API_BLOCK_GOSSIP_TIMES: LazyLock<Result<HistogramVec>> = LazyLock::new(|| {
    try_create_histogram_vec_with_buckets(
        "http_api_block_gossip_times",
        "Time between receiving the block on HTTP and publishing it on gossip",
        decimal_buckets(-2, 2),
        &["provenance"],
    )
});
pub static HTTP_API_STATE_SSZ_ENCODE_TIMES: LazyLock<Result<Histogram>> = LazyLock::new(|| {
    try_create_histogram(
        "http_api_state_ssz_encode_times",
        "Time to SSZ encode a BeaconState for a response",
    )
});
pub static HTTP_API_STATE_ROOT_TIMES: LazyLock<Result<Histogram>> = LazyLock::new(|| {
    try_create_histogram(
        "http_api_state_root_times",
        "Time to load a state root for a request",
    )
});
pub static INCLUSION_LISTS_PUBLISHED_TOTAL: LazyLock<Result<IntCounter>> = LazyLock::new(|| {
    try_create_int_counter(
        "beacon_inclusion_lists_published_total",
        "Total number of published inclusion lists",
    )
});

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
    fn block_broadcast_delay_histogram_covers_late_blocks() {
        // A `HistogramVec` is only gathered once it has a child.
        assert!(get_histogram(&HTTP_API_BLOCK_BROADCAST_DELAY_TIMES, &["provenance"]).is_some());

        let name = "http_api_block_broadcast_delay_times";
        let bounds = histogram_bucket_bounds(name);
        assert_ne!(bounds, DEFAULT_BUCKETS, "{name} uses the default buckets");
        // 4s is the mainnet attestation deadline and 12s is one mainnet slot.
        assert!(
            bounds.contains(&4.0),
            "{name} buckets {bounds:?} have no 4s bound"
        );
        assert!(
            bounds.last().is_some_and(|top| *top > 12.0),
            "{name} buckets {bounds:?} stop within one slot"
        );
    }
}
