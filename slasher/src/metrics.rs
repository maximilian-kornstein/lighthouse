pub use metrics::*;
use std::sync::LazyLock;

pub static SLASHER_DATABASE_SIZE: LazyLock<Result<IntGauge>> = LazyLock::new(|| {
    try_create_int_gauge(
        "slasher_database_size",
        "Size of the database backing the slasher, in bytes",
    )
});
pub static SLASHER_RUN_TIME: LazyLock<Result<Histogram>> = LazyLock::new(|| {
    try_create_histogram_with_buckets(
        "slasher_process_batch_time",
        "Time taken to process a batch of blocks and attestations",
        // 1.5s is the default slack before the next slot and 12s the default update period.
        Ok(vec![
            0.1, 0.25, 0.5, 1.0, 1.5, 2.0, 4.0, 8.0, 12.0, 24.0, 48.0, 120.0,
        ]),
    )
});
pub static SLASHER_NUM_ATTESTATIONS_DROPPED: LazyLock<Result<IntGauge>> = LazyLock::new(|| {
    try_create_int_gauge(
        "slasher_num_attestations_dropped",
        "Number of attestations dropped per batch",
    )
});
pub static SLASHER_NUM_ATTESTATIONS_DEFERRED: LazyLock<Result<IntGauge>> = LazyLock::new(|| {
    try_create_int_gauge(
        "slasher_num_attestations_deferred",
        "Number of attestations deferred per batch",
    )
});
pub static SLASHER_NUM_ATTESTATIONS_VALID: LazyLock<Result<IntGauge>> = LazyLock::new(|| {
    try_create_int_gauge(
        "slasher_num_attestations_valid",
        "Number of valid attestations per batch",
    )
});
pub static SLASHER_NUM_ATTESTATIONS_STORED_PER_BATCH: LazyLock<Result<IntGauge>> =
    LazyLock::new(|| {
        try_create_int_gauge(
            "slasher_num_attestations_stored_per_batch",
            "Number of attestations stored per batch",
        )
    });
pub static SLASHER_NUM_BLOCKS_PROCESSED: LazyLock<Result<IntGauge>> = LazyLock::new(|| {
    try_create_int_gauge(
        "slasher_num_blocks_processed",
        "Number of blocks processed per batch",
    )
});
pub static SLASHER_NUM_CHUNKS_UPDATED: LazyLock<Result<IntCounterVec>> = LazyLock::new(|| {
    try_create_int_counter_vec(
        "slasher_num_chunks_updated",
        "Number of min or max target chunks updated on disk",
        &["array"],
    )
});
pub static SLASHER_COMPRESSION_RATIO: LazyLock<Result<Gauge>> = LazyLock::new(|| {
    try_create_float_gauge(
        "slasher_compression_ratio",
        "Compression ratio for min-max array chunks (higher is better)",
    )
});
pub static SLASHER_NUM_ATTESTATION_ROOT_QUERIES: LazyLock<Result<IntCounter>> =
    LazyLock::new(|| {
        try_create_int_counter(
            "slasher_num_attestation_root_queries",
            "Number of requests for an attestation data root",
        )
    });
pub static SLASHER_NUM_ATTESTATION_ROOT_HITS: LazyLock<Result<IntCounter>> = LazyLock::new(|| {
    try_create_int_counter(
        "slasher_num_attestation_root_hits",
        "Number of requests for an attestation data root that hit the LRU cache",
    )
});
pub static SLASHER_ATTESTATION_ROOT_CACHE_SIZE: LazyLock<Result<IntGauge>> = LazyLock::new(|| {
    try_create_int_gauge(
        "slasher_attestation_root_cache_size",
        "Number of attestation data roots cached in memory",
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
    fn batch_time_histogram_covers_slow_batches() {
        assert!(SLASHER_RUN_TIME.is_ok());

        let name = "slasher_process_batch_time";
        let bounds = histogram_bucket_bounds(name);
        assert_ne!(bounds, DEFAULT_BUCKETS, "{name} uses the default buckets");
        // Batches should finish within the default 12s update period; slower ones must be visible.
        assert!(
            bounds.contains(&12.0),
            "{name} buckets {bounds:?} have no bound at the default update period"
        );
        assert!(
            bounds.last().is_some_and(|top| *top >= 60.0),
            "{name} buckets {bounds:?} stop below 60s"
        );
    }
}
