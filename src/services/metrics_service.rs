// System metrics collection service
// Handles gathering metrics using sysinfo crate with caching and error handling

use crate::models::{
    MetricsCollectionError, MetricsResponse, ServerMetrics, MemoryMetrics, 
    CpuMetrics, NetworkMetrics, OsInfo
};
use crate::models::cpu_metrics::LoadAverage;
use chrono::Utc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use tokio::sync::Mutex;
use tracing::{debug, error, instrument};

/// Normalize OS name to standard identifiers
fn normalize_os_name(raw_name: &str, distribution: Option<&str>) -> String {
    let name_lower = raw_name.to_lowercase();
    
    // Linux distributions
    if name_lower.contains("ubuntu") || name_lower.contains("debian") || 
       name_lower.contains("fedora") || name_lower.contains("centos") ||
       name_lower.contains("rhel") || name_lower.contains("suse") ||
       name_lower.contains("arch") || name_lower.contains("mint") ||
       (distribution.is_some() && !name_lower.contains("windows") && !name_lower.contains("macos")) {
        return "Linux".to_string();
    }
    
    // Windows variants
    if name_lower.contains("windows") {
        return "Windows".to_string();
    }
    
    // macOS variants
    if name_lower.contains("macos") || name_lower.contains("darwin") || name_lower.contains("osx") {
        return "macOS".to_string();
    }
    
    // BSD variants
    if name_lower.contains("freebsd") || name_lower.contains("openbsd") || name_lower.contains("netbsd") {
        return "FreeBSD".to_string();
    }
    
    // If we can't identify it, return the original name
    raw_name.to_string()
}

/// Configuration for metrics collection service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsServiceConfig {
    /// Collection interval in seconds
    pub collection_interval_seconds: u32,
    /// Cache duration for metrics data
    pub cache_duration_seconds: u32,
    /// Timeout for individual metric collection operations
    pub collection_timeout_ms: u64,
    /// Maximum number of cached entries (for memory management)
    pub max_cache_entries: usize,
    /// Whether to collect network interface metrics
    pub collect_network_metrics: bool,
    /// Whether to collect detailed CPU metrics per core
    pub collect_cpu_per_core: bool,
}

impl Default for MetricsServiceConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: 5,
            cache_duration_seconds: 3,
            collection_timeout_ms: 2000,
            max_cache_entries: 100,
            collect_network_metrics: true,
            collect_cpu_per_core: true,
        }
    }
}

/// Cached metrics entry
#[derive(Debug, Clone)]
struct CachedMetrics {
    #[allow(dead_code)]
    metrics: ServerMetrics,
    #[allow(dead_code)]
    cached_at: Instant,
    #[allow(dead_code)]
    collection_duration_ms: u64,
}

impl CachedMetrics {
    fn new(metrics: ServerMetrics, collection_duration_ms: u64) -> Self {
        Self {
            metrics,
            cached_at: Instant::now(),
            collection_duration_ms,
        }
    }

    #[allow(dead_code)]
    fn is_expired(&self, cache_duration: Duration) -> bool {
        self.cached_at.elapsed() > cache_duration
    }
}

/// Service for collecting system metrics
pub struct MetricsService {
    config: MetricsServiceConfig,
    system: Arc<Mutex<System>>,
    cache: Arc<RwLock<Option<CachedMetrics>>>,
    collection_stats: Arc<RwLock<CollectionStats>>,
}

/// Statistics about metrics collection performance
#[derive(Debug, Clone, Default)]
pub struct CollectionStats {
    pub total_collections: u64,
    pub successful_collections: u64,
    pub failed_collections: u64,
    #[allow(dead_code)]
    pub cache_hits: u64,
    #[allow(dead_code)]
    pub cache_misses: u64,
    pub average_collection_time_ms: f64,
    pub last_error: Option<MetricsCollectionError>,
}

impl MetricsService {
    /// Create a new metrics service with default configuration
    pub fn new() -> Self {
        Self::with_config(MetricsServiceConfig::default())
    }

    /// Create a new metrics service with custom configuration
    pub fn with_config(config: MetricsServiceConfig) -> Self {
        let system = System::new_with_specifics(RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
        );

        Self {
            config,
            system: Arc::new(Mutex::new(system)),
            cache: Arc::new(RwLock::new(None)),
            collection_stats: Arc::new(RwLock::new(CollectionStats::default())),
        }
    }

    /// Initialize the service by performing an initial system refresh
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<(), MetricsCollectionError> {
        let mut system = self.system.lock().await;
        
        // Initial refresh to populate system information
        system.refresh_all();
        
        // Wait a bit for CPU usage calculation to stabilize
        tokio::time::sleep(Duration::from_millis(100)).await;
        system.refresh_cpu();
        
        debug!("MetricsService initialized successfully");
        Ok(())
    }

    /// Get current server metrics (uses cache if available and fresh)
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn get_metrics(&self) -> MetricsResponse<ServerMetrics> {
        // Check cache first
        if let Some(cached) = self.get_from_cache().await {
            self.update_stats(|stats| stats.cache_hits += 1).await;
            debug!("Returning cached metrics");
            return MetricsResponse::Ok(cached.metrics);
        }

        // Cache miss - collect fresh metrics
        self.update_stats(|stats| stats.cache_misses += 1).await;
        self.collect_fresh_metrics().await
    }

    /// Force collection of fresh metrics (bypasses cache)
    #[instrument(skip(self))]
    pub async fn collect_fresh_metrics(&self) -> MetricsResponse<ServerMetrics> {
        let start_time = Instant::now();
        
        self.update_stats(|stats| stats.total_collections += 1).await;

        let result = self.perform_collection().await;
        let collection_duration = start_time.elapsed().as_millis() as u64;

        match &result {
            MetricsResponse::Ok(metrics) | MetricsResponse::PartialData { data: metrics, .. } => {
                // Cache successful result
                let cached = CachedMetrics::new(metrics.clone(), collection_duration);
                *self.cache.write().unwrap() = Some(cached);
                
                self.update_stats(|stats| {
                    stats.successful_collections += 1;
                    stats.average_collection_time_ms = 
                        (stats.average_collection_time_ms * (stats.successful_collections - 1) as f64 + collection_duration as f64) 
                        / stats.successful_collections as f64;
                }).await;

                debug!("Metrics collected successfully in {}ms", collection_duration);
            }
            MetricsResponse::Error(error) => {
                self.update_stats(|stats| {
                    stats.failed_collections += 1;
                    stats.last_error = Some(error.clone());
                }).await;

                error!("Failed to collect metrics: {}", error);
            }
        }

        result
    }

    /// Collect OS information independently
    #[instrument(skip(self))]
    pub async fn collect_os_info(&self) -> Result<OsInfo, MetricsCollectionError> {
        let system = self.system.lock().await;
        self.collect_os_info_from_system(&system)
    }

    /// Perform the actual metrics collection
    async fn perform_collection(&self) -> MetricsResponse<ServerMetrics> {
        let mut errors = Vec::new();
        let collection_time = Utc::now();

        // Refresh system information
        let mut system = self.system.lock().await;
        system.refresh_all();

        // Collect memory metrics
        let memory_metrics = match self.collect_memory_metrics(&system) {
            Ok(metrics) => metrics,
            Err(error) => {
                errors.push(error);
                MemoryMetrics::default() // Use default if collection fails
            }
        };

        // Collect CPU metrics
        let cpu_metrics = match self.collect_cpu_metrics(&system) {
            Ok(metrics) => metrics,
            Err(error) => {
                errors.push(error);
                CpuMetrics::default() // Use default if collection fails
            }
        };

        // Collect network metrics
        let network_metrics = if self.config.collect_network_metrics {
            match self.collect_network_metrics(&system) {
                Ok(metrics) => metrics,
                Err(error) => {
                    errors.push(error);
                    NetworkMetrics::default() // Use default if collection fails
                }
            }
        } else {
            NetworkMetrics::default()
        };

        // Get system uptime using sysinfo 0.30 API
        let uptime = match sysinfo::System::uptime() {
            uptime_secs if uptime_secs > 0 => Duration::from_secs(uptime_secs),
            _ => {
                let error = MetricsCollectionError::system_unavailable("uptime not available");
                errors.push(error);
                Duration::from_secs(0)
            }
        };

        // Check if we have meaningful data before creating server metrics
        let has_meaningful_data = memory_metrics.total_bytes > 0 || cpu_metrics.usage_percentage > 0.0;

        // Create server metrics
        let server_metrics = ServerMetrics {
            timestamp: collection_time,
            memory_usage: memory_metrics,
            cpu_usage: cpu_metrics,
            uptime,
            network_metrics,
        };

        // Return appropriate response based on errors
        if errors.is_empty() {
            MetricsResponse::Ok(server_metrics)
        } else if errors.len() == 1 {
            // Single error but we have some data
            MetricsResponse::PartialData {
                data: server_metrics,
                errors,
            }
        } else {
            // Multiple errors - check if we have any meaningful data
            if !has_meaningful_data {
                // No meaningful data collected
                MetricsResponse::Error(MetricsCollectionError::multiple(errors))
            } else {
                // Some meaningful data collected despite errors
                MetricsResponse::PartialData {
                    data: server_metrics,
                    errors,
                }
            }
        }
    }

    /// Collect memory metrics from system
    fn collect_memory_metrics(&self, system: &System) -> Result<MemoryMetrics, MetricsCollectionError> {
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = system.available_memory();
        let _total_swap = system.total_swap();
        let _used_swap = system.used_swap();

        if total_memory == 0 {
            return Err(MetricsCollectionError::memory_error("total memory is zero"));
        }

        Ok(MemoryMetrics {
            total_bytes: total_memory,
            used_bytes: used_memory,
            available_bytes: available_memory,
            usage_percentage: ((used_memory as f64 / total_memory as f64) * 100.0) as f32,
        })
    }

    /// Collect CPU metrics from system
    fn collect_cpu_metrics(&self, system: &System) -> Result<CpuMetrics, MetricsCollectionError> {
        let cpus = system.cpus();
        
        if cpus.is_empty() {
            return Err(MetricsCollectionError::cpu_error("no CPUs detected"));
        }

        let overall_usage = system.global_cpu_info().cpu_usage();
        
        let _per_core_usage = if self.config.collect_cpu_per_core {
            cpus.iter().map(|cpu| cpu.cpu_usage()).collect()
        } else {
            vec![]
        };

        let system_load_average = sysinfo::System::load_average();
        let load_average = LoadAverage {
            one_minute: system_load_average.one as f32,
            five_minute: system_load_average.five as f32,
            fifteen_minute: system_load_average.fifteen as f32,
        };

        Ok(CpuMetrics {
            usage_percentage: overall_usage,
            core_count: cpus.len() as u32,
            load_average,
        })
    }

    /// Collect network metrics from system
    fn collect_network_metrics(&self, _system: &System) -> Result<NetworkMetrics, MetricsCollectionError> {
        // Read network statistics from /proc/net/dev on Linux
        use std::fs;
        
        let mut total_bytes_sent = 0;
        let mut total_bytes_received = 0;
        let mut total_packets_sent = 0;
        let mut total_packets_received = 0;

        if let Ok(contents) = fs::read_to_string("/proc/net/dev") {
            for line in contents.lines().skip(2) { // Skip header lines
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 17 {
                    let interface = parts[0].trim_end_matches(':');
                    
                    // Skip loopback and virtual interfaces
                    if interface.starts_with("lo") || interface.starts_with("docker") || 
                       interface.starts_with("br-") || interface.starts_with("veth") {
                        continue;
                    }

                    if let (Ok(rx_bytes), Ok(rx_packets), Ok(tx_bytes), Ok(tx_packets)) = (
                        parts[1].parse::<u64>(),  // received bytes
                        parts[2].parse::<u64>(),  // received packets
                        parts[9].parse::<u64>(),  // transmitted bytes
                        parts[10].parse::<u64>(), // transmitted packets
                    ) {
                        total_bytes_received += rx_bytes;
                        total_packets_received += rx_packets;
                        total_bytes_sent += tx_bytes;
                        total_packets_sent += tx_packets;
                    }
                }
            }
        }

        // Get active connections count
        let active_connections = self.estimate_active_connections();

        Ok(NetworkMetrics {
            bytes_sent: total_bytes_sent,
            bytes_received: total_bytes_received,
            packets_sent: total_packets_sent,
            packets_received: total_packets_received,
            active_connections,
        })
    }

    /// Estimate active network connections
    fn estimate_active_connections(&self) -> u32 {
        use std::fs;
        
        // Count TCP connections in ESTABLISHED state
        let mut count = 0;
        
        if let Ok(contents) = fs::read_to_string("/proc/net/tcp") {
            for line in contents.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 && parts[3] == "01" { // 01 = ESTABLISHED
                    count += 1;
                }
            }
        }
        
        // Also count IPv6 connections
        if let Ok(contents) = fs::read_to_string("/proc/net/tcp6") {
            for line in contents.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 && parts[3] == "01" { // 01 = ESTABLISHED
                    count += 1;
                }
            }
        }
        
        count
    }

    /// Collect OS information from system
    fn collect_os_info_from_system(&self, _system: &System) -> Result<OsInfo, MetricsCollectionError> {
        // Get raw OS name using static methods
        let raw_name = match System::name() {
            Some(name) if !name.trim().is_empty() => name,
            _ => return Err(MetricsCollectionError::system_unavailable("OS name not available")),
        };

        // Get distribution info (for Linux systems)
        let distribution = match System::distribution_id() {
            d if !d.trim().is_empty() => Some(d),
            _ => None,
        };

        // Normalize OS name based on known patterns
        let name = normalize_os_name(&raw_name, distribution.as_deref());

        // Get OS version 
        let version = match System::os_version() {
            Some(version) if !version.trim().is_empty() => version,
            _ => "Unknown".to_string(),
        };

        // Get architecture
        let architecture = match System::cpu_arch() {
            Some(arch) if !arch.trim().is_empty() => arch,
            _ => "Unknown".to_string(),
        };

        // Get kernel version
        let kernel_version = match System::kernel_version() {
            Some(kernel) if !kernel.trim().is_empty() => kernel,
            _ => "Unknown".to_string(),
        };

        // Create long description by combining available info
        let long_description = if let Some(dist) = &distribution {
            format!("{} {} {} ({})", name, dist, version, architecture)
        } else {
            format!("{} {} ({})", name, version, architecture)
        };

        // Create OsInfo using struct syntax and validate
        let os_info = OsInfo {
            name,
            version,
            architecture,
            kernel_version,
            distribution,
            long_description,
        };

        // Validate the created OsInfo
        match os_info.validate() {
            Ok(()) => Ok(os_info),
            Err(validation_error) => {
                // If validation fails, create fallback OsInfo
                tracing::warn!("OS info validation failed: {}, using fallback", validation_error);
                Ok(OsInfo::fallback())
            }
        }
    }

    /// Get metrics from cache if available and fresh
    #[allow(dead_code)]
    async fn get_from_cache(&self) -> Option<CachedMetrics> {
        let cache = self.cache.read().unwrap();
        if let Some(ref cached) = *cache {
            let cache_duration = Duration::from_secs(self.config.cache_duration_seconds as u64);
            if !cached.is_expired(cache_duration) {
                return Some(cached.clone());
            }
        }
        None
    }

    /// Update collection statistics
    async fn update_stats<F>(&self, updater: F) 
    where
        F: FnOnce(&mut CollectionStats),
    {
        let mut stats = self.collection_stats.write().unwrap();
        updater(&mut *stats);
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> CollectionStats {
        self.collection_stats.read().unwrap().clone()
    }

    /// Get service configuration
    pub fn get_config(&self) -> &MetricsServiceConfig {
        &self.config
    }

    /// Clear the metrics cache
    #[allow(dead_code)]
    pub async fn clear_cache(&self) {
        *self.cache.write().unwrap() = None;
        debug!("Metrics cache cleared");
    }

    /// Update service configuration
    #[allow(dead_code)]
    pub async fn update_config(&mut self, new_config: MetricsServiceConfig) {
        self.config = new_config;
        // Clear cache since configuration changed
        self.clear_cache().await;
        debug!("MetricsService configuration updated");
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_metrics_service_creation() {
        let service = MetricsService::new();
        assert_eq!(service.config.collection_interval_seconds, 5);
        assert_eq!(service.config.cache_duration_seconds, 3);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = MetricsServiceConfig {
            collection_interval_seconds: 10,
            cache_duration_seconds: 5,
            collection_timeout_ms: 5000,
            max_cache_entries: 50,
            collect_network_metrics: false,
            collect_cpu_per_core: false,
        };

        let service = MetricsService::with_config(config.clone());
        assert_eq!(service.config.collection_interval_seconds, 10);
        assert!(!service.config.collect_network_metrics);
    }

    #[tokio::test]
    async fn test_service_initialization() {
        let service = MetricsService::new();
        let result = service.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let service = MetricsService::new();
        service.initialize().await.unwrap();

        let response = service.get_metrics().await;
        
        match response {
            MetricsResponse::Ok(metrics) | MetricsResponse::PartialData { data: metrics, .. } => {
                // Verify we got some meaningful data
                assert!(metrics.memory_usage.total_bytes > 0);
                assert!(metrics.cpu_usage.core_count > 0);
                assert!(!metrics.timestamp.naive_utc().format("%Y").to_string().is_empty());
            }
            MetricsResponse::Error(error) => {
                // On some systems, metrics collection might fail
                println!("Metrics collection failed (expected on some test environments): {}", error);
            }
        }
    }

    #[tokio::test]
    async fn test_metrics_caching() {
        let mut config = MetricsServiceConfig::default();
        config.cache_duration_seconds = 2; // 2 second cache

        let service = MetricsService::with_config(config);
        service.initialize().await.unwrap();

        // First call - should be cache miss
        let response1 = service.get_metrics().await;
        let stats1 = service.get_stats().await;
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.cache_hits, 0);

        // Second call immediately - should be cache hit
        let response2 = service.get_metrics().await;
        let stats2 = service.get_stats().await;
        assert_eq!(stats2.cache_hits, 1);
        assert_eq!(stats2.cache_misses, 1);

        // Verify both responses have data
        assert!(response1.has_data());
        assert!(response2.has_data());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mut config = MetricsServiceConfig::default();
        config.cache_duration_seconds = 1; // 1 second cache

        let service = MetricsService::with_config(config);
        service.initialize().await.unwrap();

        // First call
        let _response1 = service.get_metrics().await;
        
        // Wait for cache to expire
        sleep(TokioDuration::from_millis(1100)).await;
        
        // Second call should be cache miss due to expiration
        let _response2 = service.get_metrics().await;
        let stats = service.get_stats().await;
        
        assert_eq!(stats.cache_misses, 2); // Both calls were cache misses
        assert_eq!(stats.cache_hits, 0);
    }

    #[tokio::test]
    async fn test_force_fresh_collection() {
        let service = MetricsService::new();
        service.initialize().await.unwrap();

        // First call to populate cache
        let _response1 = service.get_metrics().await;
        
        // Force fresh collection should bypass cache
        let response2 = service.collect_fresh_metrics().await;
        let stats = service.get_stats().await;

        // Should have 1 cache miss (fresh collection)
        assert_eq!(stats.cache_misses, 1);
        assert!(response2.has_data());
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let service = MetricsService::new();
        service.initialize().await.unwrap();

        // Populate cache
        let _response1 = service.get_metrics().await;
        
        // Clear cache
        service.clear_cache().await;
        
        // Next call should be cache miss
        let _response2 = service.get_metrics().await;
        let stats = service.get_stats().await;
        
        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.cache_hits, 0);
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let service = MetricsService::new();
        service.initialize().await.unwrap();

        // Collect some metrics
        let _response1 = service.get_metrics().await;
        let _response2 = service.get_metrics().await; // Cache hit
        let _response3 = service.collect_fresh_metrics().await; // Force fresh

        let stats = service.get_stats().await;
        
        assert_eq!(stats.total_collections, 2); // First call + force fresh
        assert!(stats.successful_collections > 0);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert!(stats.average_collection_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_config_update() {
        let mut service = MetricsService::new();
        service.initialize().await.unwrap();

        // Populate cache
        let _response1 = service.get_metrics().await;
        
        // Update configuration
        let new_config = MetricsServiceConfig {
            collection_interval_seconds: 15,
            cache_duration_seconds: 10,
            ..Default::default()
        };
        
        service.update_config(new_config).await;
        
        // Configuration should be updated and cache cleared
        assert_eq!(service.config.collection_interval_seconds, 15);
        
        // Next call should be cache miss due to cache clear
        let _response2 = service.get_metrics().await;
        let stats = service.get_stats().await;
        assert_eq!(stats.cache_misses, 2);
    }

    #[tokio::test]
    async fn test_os_info_collection() {
        let service = MetricsService::new();
        service.initialize().await.unwrap();

        // Test OS info collection
        let os_info_result = service.collect_os_info().await;
        assert!(os_info_result.is_ok(), "OS info collection should succeed");

        let os_info = os_info_result.unwrap();
        
        // OS info should have meaningful data (not fallback values)
        assert!(!os_info.name.is_empty(), "OS name should not be empty");
        assert!(os_info.name != "Unknown", "Should collect real OS name, not fallback");
        assert!(!os_info.version.is_empty(), "OS version should not be empty");
        assert!(!os_info.architecture.is_empty(), "OS architecture should not be empty");
        assert!(!os_info.kernel_version.is_empty(), "Kernel version should not be empty");
        assert!(!os_info.long_description.is_empty(), "Long description should not be empty");
        
        // Validate the collected OS info
        assert!(os_info.validate().is_ok(), "Collected OS info should be valid");
    }
}