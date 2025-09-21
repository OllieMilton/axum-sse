// Advanced caching layer for metrics data
// Provides LRU cache with TTL, background refresh, and performance optimization

use crate::models::{ServerMetrics, MetricsCollectionError, MetricsResponse};
use crate::services::MetricsService;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock as TokioRwLock};
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, warn, error, instrument};

/// Configuration for the metrics cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCacheConfig {
    /// Maximum number of cached entries
    pub max_entries: usize,
    /// Time-to-live for cache entries in seconds
    pub ttl_seconds: u32,
    /// Background refresh interval in seconds
    pub background_refresh_interval_seconds: u32,
    /// Whether to enable background refresh
    pub enable_background_refresh: bool,
    /// Prefetch threshold - refresh when TTL has this percentage remaining
    pub prefetch_threshold_percent: f64,
    /// Maximum concurrent background refresh operations
    pub max_concurrent_refreshes: usize,
}

impl Default for MetricsCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_seconds: 30,
            background_refresh_interval_seconds: 10,
            enable_background_refresh: true,
            prefetch_threshold_percent: 0.2, // Refresh when 20% of TTL remains
            max_concurrent_refreshes: 3,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: ServerMetrics,
    pub created_at: Instant,
    #[allow(dead_code)]
    pub accessed_at: Instant,
    #[allow(dead_code)]
    pub access_count: u64,
    #[allow(dead_code)]
    pub cache_key: String,
    #[allow(dead_code)]
    pub collection_time_ms: u64,
}

impl CacheEntry {
    fn new(data: ServerMetrics, cache_key: String, collection_time_ms: u64) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            accessed_at: now,
            access_count: 1,
            cache_key,
            collection_time_ms,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    fn should_prefetch(&self, ttl: Duration, threshold: f64) -> bool {
        let elapsed = self.created_at.elapsed();
        let remaining_ratio = 1.0 - (elapsed.as_secs_f64() / ttl.as_secs_f64());
        remaining_ratio <= threshold && remaining_ratio > 0.0
    }

    #[allow(dead_code)]
    fn touch(&mut self) {
        self.accessed_at = Instant::now();
        self.access_count += 1;
    }
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub background_refreshes: u64,
    pub failed_refreshes: u64,
    pub current_entries: usize,
    pub average_collection_time_ms: f64,
    pub hit_ratio: f64,
}

impl CacheStats {
    fn calculate_hit_ratio(&mut self) {
        if self.total_requests > 0 {
            self.hit_ratio = self.cache_hits as f64 / self.total_requests as f64;
        }
    }
}

/// Advanced metrics cache with LRU eviction and background refresh
pub struct MetricsCache {
    config: MetricsCacheConfig,
    cache: Arc<TokioRwLock<HashMap<String, CacheEntry>>>,
    access_order: Arc<Mutex<VecDeque<String>>>,
    stats: Arc<RwLock<CacheStats>>,
    metrics_service: Arc<MetricsService>,
    background_refresh_active: Arc<Mutex<bool>>,
}

impl MetricsCache {
    /// Create a new metrics cache with default configuration
    pub fn new(metrics_service: Arc<MetricsService>) -> Self {
        Self::with_config(MetricsCacheConfig::default(), metrics_service)
    }

    /// Create a new metrics cache with custom configuration
    pub fn with_config(config: MetricsCacheConfig, metrics_service: Arc<MetricsService>) -> Self {
        Self {
            config,
            cache: Arc::new(TokioRwLock::new(HashMap::new())),
            access_order: Arc::new(Mutex::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            metrics_service,
            background_refresh_active: Arc::new(Mutex::new(false)),
        }
    }

    /// Start background refresh task
    #[instrument(skip(self))]
    pub async fn start_background_refresh(&self) -> Result<(), MetricsCollectionError> {
        if !self.config.enable_background_refresh {
            debug!("Background refresh is disabled");
            return Ok(());
        }

        let mut active = self.background_refresh_active.lock().await;
        if *active {
            debug!("Background refresh already active");
            return Ok(());
        }
        *active = true;
        drop(active);

        let cache = Arc::clone(&self.cache);
        let stats = Arc::clone(&self.stats);
        let metrics_service = Arc::clone(&self.metrics_service);
        let config = self.config.clone();
        let background_active = Arc::clone(&self.background_refresh_active);

        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(
                config.background_refresh_interval_seconds as u64
            ));
            interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

            debug!("Background refresh task started");

            loop {
                interval_timer.tick().await;

                // Check if background refresh is still active
                {
                    let active = background_active.lock().await;
                    if !*active {
                        debug!("Background refresh task stopping");
                        break;
                    }
                }

                // Find entries that need refresh
                let entries_to_refresh = {
                    let cache = cache.read().await;
                    let ttl = Duration::from_secs(config.ttl_seconds as u64);
                    
                    cache.iter()
                        .filter(|(_, entry)| {
                            entry.should_prefetch(ttl, config.prefetch_threshold_percent)
                        })
                        .take(config.max_concurrent_refreshes)
                        .map(|(key, _)| key.clone())
                        .collect::<Vec<_>>()
                };

                if !entries_to_refresh.is_empty() {
                    debug!("Background refreshing {} cache entries", entries_to_refresh.len());
                    
                    // Refresh entries in parallel
                    let refresh_tasks = entries_to_refresh.into_iter().map(|key| {
                        let cache_clone = Arc::clone(&cache);
                        let stats_clone = Arc::clone(&stats);
                        let service_clone = Arc::clone(&metrics_service);
                        let key_clone = key.clone();
                        
                        tokio::spawn(async move {
                            match service_clone.collect_fresh_metrics().await {
                                MetricsResponse::Ok(metrics) | MetricsResponse::PartialData { data: metrics, .. } => {
                                    let mut cache = cache_clone.write().await;
                                    if let Some(entry) = cache.get_mut(&key_clone) {
                                        entry.data = metrics;
                                        entry.created_at = Instant::now();
                                        
                                        let mut stats = stats_clone.write().unwrap();
                                        stats.background_refreshes += 1;
                                        
                                        debug!("Background refreshed cache entry: {}", key_clone);
                                    }
                                }
                                MetricsResponse::Error(error) => {
                                    warn!("Background refresh failed for {}: {}", key_clone, error);
                                    let mut stats = stats_clone.write().unwrap();
                                    stats.failed_refreshes += 1;
                                }
                            }
                        })
                    });

                    // Wait for all refresh tasks to complete
                    for task in refresh_tasks {
                        if let Err(e) = task.await {
                            error!("Background refresh task failed: {}", e);
                        }
                    }
                }
            }
        });

        debug!("Background refresh task initialized");
        Ok(())
    }

    /// Stop background refresh task
    #[allow(dead_code)]
    pub async fn stop_background_refresh(&self) {
        let mut active = self.background_refresh_active.lock().await;
        *active = false;
        debug!("Background refresh task stopped");
    }

    /// Get metrics from cache or collect fresh if not available
    #[instrument(skip(self))]
    pub async fn get_metrics(&self, cache_key: Option<String>) -> MetricsResponse<ServerMetrics> {
        let key = cache_key.unwrap_or_else(|| "default".to_string());
        
        self.update_stats(|stats| stats.total_requests += 1);

        // Try to get from cache first
        if let Some(metrics) = self.get_from_cache(&key).await {
            self.update_stats(|stats| {
                stats.cache_hits += 1;
                stats.calculate_hit_ratio();
            });
            debug!("Cache hit for key: {}", key);
            return MetricsResponse::Ok(metrics);
        }

        // Cache miss - collect fresh metrics
        self.update_stats(|stats| {
            stats.cache_misses += 1;
            stats.calculate_hit_ratio();
        });
        
        debug!("Cache miss for key: {}", key);
        let start_time = Instant::now();
        
        let result = self.metrics_service.collect_fresh_metrics().await;
        let collection_time = start_time.elapsed().as_millis() as u64;

        // Cache the result if successful
        match &result {
            MetricsResponse::Ok(metrics) | MetricsResponse::PartialData { data: metrics, .. } => {
                self.put_in_cache(key.clone(), metrics.clone(), collection_time).await;
                self.update_stats(|stats| {
                    stats.average_collection_time_ms = if stats.cache_misses == 1 {
                        collection_time as f64
                    } else {
                        (stats.average_collection_time_ms * (stats.cache_misses - 1) as f64 + collection_time as f64) / stats.cache_misses as f64
                    };
                });
                debug!("Cached fresh metrics for key: {}", key);
            }
            MetricsResponse::Error(error) => {
                warn!("Failed to collect metrics for caching: {}", error);
            }
        }

        result
    }

    /// Get metrics from cache if available and not expired
    async fn get_from_cache(&self, key: &str) -> Option<ServerMetrics> {
        let cache = self.cache.read().await;
        
        if let Some(entry) = cache.get(key) {
            let ttl = Duration::from_secs(self.config.ttl_seconds as u64);
            
            if !entry.is_expired(ttl) {
                // Update access order
                self.update_access_order(key.to_string()).await;
                
                // Return cloned data
                return Some(entry.data.clone());
            } else {
                debug!("Cache entry expired for key: {}", key);
            }
        }
        
        None
    }

    /// Put metrics in cache
    async fn put_in_cache(&self, key: String, metrics: ServerMetrics, collection_time_ms: u64) {
        let mut cache = self.cache.write().await;
        
        // Check if we need to evict entries
        if cache.len() >= self.config.max_entries {
            self.evict_lru_entries(&mut cache).await;
        }

        // Create new cache entry
        let entry = CacheEntry::new(metrics, key.clone(), collection_time_ms);
        cache.insert(key.clone(), entry);
        
        // Update access order
        self.update_access_order(key).await;
        
        // Update stats
        self.update_stats(|stats| {
            stats.current_entries = cache.len();
        });
    }

    /// Evict least recently used entries
    async fn evict_lru_entries(&self, cache: &mut HashMap<String, CacheEntry>) {
        let mut access_order = self.access_order.lock().await;
        
        // Calculate how many entries to evict (25% of max)
        let evict_count = (self.config.max_entries / 4).max(1);
        
        for _ in 0..evict_count {
            if let Some(lru_key) = access_order.pop_front() {
                if cache.remove(&lru_key).is_some() {
                    self.update_stats(|stats| stats.evictions += 1);
                    debug!("Evicted LRU cache entry: {}", lru_key);
                }
            } else {
                break;
            }
        }
    }

    /// Update access order for LRU tracking
    async fn update_access_order(&self, key: String) {
        let mut access_order = self.access_order.lock().await;
        
        // Remove existing entry if present
        if let Some(pos) = access_order.iter().position(|k| k == &key) {
            access_order.remove(pos);
        }
        
        // Add to back (most recently used)
        access_order.push_back(key);
    }

    /// Update cache statistics
    fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut CacheStats),
    {
        let mut stats = self.stats.write().unwrap();
        updater(&mut *stats);
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.read().unwrap().clone();
        
        // Update current entries count
        if let Ok(cache) = self.cache.try_read() {
            stats.current_entries = cache.len();
        }
        
        stats
    }

    /// Clear all cache entries
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut access_order = self.access_order.lock().await;
        
        cache.clear();
        access_order.clear();
        
        self.update_stats(|stats| {
            stats.current_entries = 0;
        });
        
        debug!("Cache cleared");
    }

    /// Remove expired entries from cache
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().await;
        let mut access_order = self.access_order.lock().await;
        let ttl = Duration::from_secs(self.config.ttl_seconds as u64);
        
        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired(ttl))
            .map(|(key, _)| key.clone())
            .collect();
        
        let expired_count = expired_keys.len();
        
        for key in expired_keys {
            cache.remove(&key);
            if let Some(pos) = access_order.iter().position(|k| k == &key) {
                access_order.remove(pos);
            }
        }
        
        if expired_count > 0 {
            self.update_stats(|stats| {
                stats.current_entries = cache.len();
                stats.evictions += expired_count as u64;
            });
            debug!("Cleaned up {} expired cache entries", expired_count);
        }
        
        expired_count
    }

    /// Get cache configuration
    #[allow(dead_code)]
    pub fn get_config(&self) -> &MetricsCacheConfig {
        &self.config
    }

    /// Update cache configuration
    #[allow(dead_code)]
    pub async fn update_config(&mut self, new_config: MetricsCacheConfig) {
        // Stop background refresh if it was enabled and is being disabled
        if self.config.enable_background_refresh && !new_config.enable_background_refresh {
            self.stop_background_refresh().await;
        }
        
        self.config = new_config;
        
        // Start background refresh if it was disabled and is being enabled
        if self.config.enable_background_refresh {
            if let Err(e) = self.start_background_refresh().await {
                error!("Failed to start background refresh after config update: {}", e);
            }
        }
        
        debug!("MetricsCache configuration updated");
    }

    /// Get all cache keys
    #[allow(dead_code)]
    pub async fn get_cache_keys(&self) -> Vec<String> {
        let cache = self.cache.read().await;
        cache.keys().cloned().collect()
    }

    /// Get cache entry details for monitoring
    #[allow(dead_code)]
    pub async fn get_cache_entry_details(&self, key: &str) -> Option<(ServerMetrics, Instant, u64)> {
        let cache = self.cache.read().await;
        cache.get(key).map(|entry| {
            (entry.data.clone(), entry.created_at, entry.access_count)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::MetricsService;
    use tokio::time::{sleep, Duration as TokioDuration};

    fn create_test_metrics_service() -> Arc<MetricsService> {
        Arc::new(MetricsService::new())
    }

    #[tokio::test]
    async fn test_cache_creation() {
        let service = create_test_metrics_service();
        let cache = MetricsCache::new(service);
        
        assert_eq!(cache.config.max_entries, 1000);
        assert_eq!(cache.config.ttl_seconds, 30);
        assert!(cache.config.enable_background_refresh);
    }

    #[tokio::test]
    async fn test_custom_cache_config() {
        let service = create_test_metrics_service();
        let config = MetricsCacheConfig {
            max_entries: 100,
            ttl_seconds: 60,
            enable_background_refresh: false,
            ..Default::default()
        };
        
        let cache = MetricsCache::with_config(config, service);
        assert_eq!(cache.config.max_entries, 100);
        assert_eq!(cache.config.ttl_seconds, 60);
        assert!(!cache.config.enable_background_refresh);
    }

    #[tokio::test]
    async fn test_cache_miss_and_hit() {
        let service = create_test_metrics_service();
        service.initialize().await.unwrap();
        
        let cache = MetricsCache::new(service);
        
        // First request should be cache miss
        let response1 = cache.get_metrics(Some("test_key".to_string())).await;
        let stats1 = cache.get_stats();
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.cache_hits, 0);
        assert!(response1.has_data());
        
        // Second request should be cache hit
        let response2 = cache.get_metrics(Some("test_key".to_string())).await;
        let stats2 = cache.get_stats();
        assert_eq!(stats2.cache_hits, 1);
        assert_eq!(stats2.cache_misses, 1);
        assert!(response2.has_data());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let service = create_test_metrics_service();
        service.initialize().await.unwrap();
        
        let config = MetricsCacheConfig {
            ttl_seconds: 1, // 1 second TTL
            enable_background_refresh: false,
            ..Default::default()
        };
        
        let cache = MetricsCache::with_config(config, service);
        
        // First request
        let _response1 = cache.get_metrics(Some("test_key".to_string())).await;
        
        // Wait for expiration
        sleep(TokioDuration::from_millis(1100)).await;
        
        // Second request should be cache miss due to expiration
        let _response2 = cache.get_metrics(Some("test_key".to_string())).await;
        let stats = cache.get_stats();
        
        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.cache_hits, 0);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let service = create_test_metrics_service();
        service.initialize().await.unwrap();
        
        let cache = MetricsCache::new(service);
        
        // Populate cache
        let _response1 = cache.get_metrics(Some("test_key".to_string())).await;
        let stats1 = cache.get_stats();
        assert_eq!(stats1.current_entries, 1);
        
        // Clear cache
        cache.clear().await;
        let stats2 = cache.get_stats();
        assert_eq!(stats2.current_entries, 0);
        
        // Next request should be cache miss
        let _response2 = cache.get_metrics(Some("test_key".to_string())).await;
        let stats3 = cache.get_stats();
        assert_eq!(stats3.cache_misses, 2);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let service = create_test_metrics_service();
        service.initialize().await.unwrap();
        
        let config = MetricsCacheConfig {
            ttl_seconds: 1,
            enable_background_refresh: false,
            ..Default::default()
        };
        
        let cache = MetricsCache::with_config(config, service);
        
        // Populate cache with multiple entries
        let _response1 = cache.get_metrics(Some("key1".to_string())).await;
        let _response2 = cache.get_metrics(Some("key2".to_string())).await;
        
        // Wait for expiration
        sleep(TokioDuration::from_millis(1100)).await;
        
        // Cleanup expired entries
        let expired_count = cache.cleanup_expired().await;
        assert_eq!(expired_count, 2);
        
        let stats = cache.get_stats();
        assert_eq!(stats.current_entries, 0);
    }

    #[tokio::test]
    async fn test_cache_keys() {
        let service = create_test_metrics_service();
        service.initialize().await.unwrap();
        
        let cache = MetricsCache::new(service);
        
        // Populate cache with multiple keys
        let _response1 = cache.get_metrics(Some("key1".to_string())).await;
        let _response2 = cache.get_metrics(Some("key2".to_string())).await;
        let _response3 = cache.get_metrics(Some("key3".to_string())).await;
        
        let keys = cache.get_cache_keys().await;
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[tokio::test]
    async fn test_cache_entry_details() {
        let service = create_test_metrics_service();
        service.initialize().await.unwrap();
        
        let cache = MetricsCache::new(service);
        
        // Populate cache
        let _response = cache.get_metrics(Some("test_key".to_string())).await;
        
        // Get entry details
        let details = cache.get_cache_entry_details("test_key").await;
        assert!(details.is_some());
        
        let (_metrics, created_at, access_count) = details.unwrap();
        assert!(created_at.elapsed() < Duration::from_secs(1));
        assert_eq!(access_count, 1);
    }

    #[tokio::test]
    async fn test_hit_ratio_calculation() {
        let service = create_test_metrics_service();
        service.initialize().await.unwrap();
        
        let cache = MetricsCache::new(service);
        
        // 1 miss, 2 hits
        let _response1 = cache.get_metrics(Some("test_key".to_string())).await;
        let _response2 = cache.get_metrics(Some("test_key".to_string())).await;
        let _response3 = cache.get_metrics(Some("test_key".to_string())).await;
        
        let stats = cache.get_stats();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert!((stats.hit_ratio - 0.6667).abs() < 0.001); // 2/3 ≈ 0.6667
    }
}