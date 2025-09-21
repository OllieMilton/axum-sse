// Integration test for metrics collection service
// Tests the system metrics collection functionality using sysinfo

use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_metrics_collection_service_basic() {
    // This test will fail until MetricsService is implemented
    
    // Create metrics service instance
    let metrics_service = create_metrics_service().await;
    
    // Collect metrics
    let result = metrics_service.collect_metrics().await;
    
    assert!(result.is_ok(), "Metrics collection should succeed");
    
    let metrics = result.unwrap();
    
    // Verify all required metrics are present
    assert!(metrics.timestamp > chrono::DateTime::from_timestamp(0, 0).unwrap(), "Timestamp should be recent");
    assert!(metrics.memory_usage.total_bytes > 0, "Total memory should be positive");
    assert!(metrics.memory_usage.used_bytes > 0, "Used memory should be positive");
    assert!(metrics.memory_usage.available_bytes > 0, "Available memory should be positive");
    assert!(metrics.memory_usage.usage_percentage >= 0.0, "Memory percentage should be non-negative");
    assert!(metrics.memory_usage.usage_percentage <= 100.0, "Memory percentage should not exceed 100%");
    
    assert!(metrics.cpu_usage.core_count > 0, "Core count should be positive");
    assert!(metrics.cpu_usage.usage_percentage >= 0.0, "CPU percentage should be non-negative");
    assert!(metrics.cpu_usage.load_average.one_minute >= 0.0, "Load average should be non-negative");
    assert!(metrics.cpu_usage.load_average.five_minute >= 0.0, "5-min load average should be non-negative");
    assert!(metrics.cpu_usage.load_average.fifteen_minute >= 0.0, "15-min load average should be non-negative");
    
    assert!(metrics.uptime.as_secs() > 0, "Uptime should be positive");
    
    assert!(metrics.network_metrics.bytes_sent >= 0, "Network bytes sent should be non-negative");
    assert!(metrics.network_metrics.bytes_received >= 0, "Network bytes received should be non-negative");
    assert!(metrics.network_metrics.packets_sent >= 0, "Network packets sent should be non-negative");
    assert!(metrics.network_metrics.packets_received >= 0, "Network packets received should be non-negative");
    assert!(metrics.network_metrics.active_connections >= 0, "Active connections should be non-negative");
}

#[tokio::test]
async fn test_metrics_collection_consistency() {
    // Test that consecutive metrics collections are consistent
    let metrics_service = create_metrics_service().await;
    
    let metrics1 = metrics_service.collect_metrics().await.unwrap();
    sleep(Duration::from_millis(100)).await; // Small delay
    let metrics2 = metrics_service.collect_metrics().await.unwrap();
    
    // Memory total should be identical
    assert_eq!(metrics1.memory_usage.total_bytes, metrics2.memory_usage.total_bytes, 
               "Total memory should be consistent");
    
    // CPU core count should be identical
    assert_eq!(metrics1.cpu_usage.core_count, metrics2.cpu_usage.core_count,
               "CPU core count should be consistent");
    
    // Uptime should increase
    assert!(metrics2.uptime >= metrics1.uptime, "Uptime should not decrease");
    
    // Timestamps should be different and in order
    assert!(metrics2.timestamp > metrics1.timestamp, "Timestamps should be in chronological order");
}

#[tokio::test]
async fn test_metrics_collection_error_handling() {
    // Test metrics collection handles system errors gracefully
    let metrics_service = create_metrics_service().await;
    
    // This should succeed even if some metrics are unavailable
    let result = metrics_service.collect_metrics().await;
    
    match result {
        Ok(metrics) => {
            // All metrics should have reasonable values even if some collection failed
            assert!(metrics.memory_usage.total_bytes > 0, "Should have fallback memory data");
        },
        Err(error) => {
            // Error should be specific and actionable
            let error_msg = format!("{:?}", error);
            assert!(!error_msg.is_empty(), "Error should have descriptive message");
        }
    }
}

#[tokio::test]
async fn test_metrics_collection_performance() {
    // Test that metrics collection is fast enough (<200ms requirement)
    let metrics_service = create_metrics_service().await;
    
    let start = std::time::Instant::now();
    let result = metrics_service.collect_metrics().await;
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Metrics collection should succeed");
    assert!(duration.as_millis() < 200, "Metrics collection should complete within 200ms, took {}ms", duration.as_millis());
}

#[tokio::test]
async fn test_metrics_validation_rules() {
    // Test that collected metrics follow validation rules
    let metrics_service = create_metrics_service().await;
    let metrics = metrics_service.collect_metrics().await.unwrap();
    
    // Memory validation rules
    let memory = &metrics.memory_usage;
    assert!(memory.used_bytes + memory.available_bytes <= memory.total_bytes,
            "Used + available memory should not exceed total");
    
    let calculated_percentage = (memory.used_bytes as f32 / memory.total_bytes as f32) * 100.0;
    let percentage_diff = (memory.usage_percentage - calculated_percentage).abs();
    assert!(percentage_diff < 1.0, "Memory percentage should match calculated value within 1%");
    
    // CPU validation rules
    let cpu = &metrics.cpu_usage;
    assert!(cpu.usage_percentage >= 0.0, "CPU usage should be non-negative");
    // Note: CPU can exceed 100% on multi-core systems
    assert!(cpu.core_count > 0, "Core count should be positive");
    
    // Timestamp freshness rule
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(metrics.timestamp);
    assert!(age.num_seconds() < 10, "Metrics timestamp should be within last 10 seconds");
}

// Helper function to create metrics service - will be implemented later
async fn create_metrics_service() -> MockMetricsService {
    // This will be replaced with actual MetricsService when implemented
    // For now, return mock to make tests fail as expected
    MockMetricsService {}
}

// Mock service for initial failing tests
struct MockMetricsService {}

impl MockMetricsService {
    async fn collect_metrics(&self) -> Result<ServerMetrics, MetricsCollectionError> {
        // This will fail until real implementation exists
        Err(MetricsCollectionError::SystemUnavailable)
    }
}

// Mock types that will be replaced with real implementations
#[derive(Debug)]
struct ServerMetrics {
    timestamp: chrono::DateTime<chrono::Utc>,
    memory_usage: MemoryMetrics,
    cpu_usage: CpuMetrics,
    uptime: Duration,
    network_metrics: NetworkMetrics,
}

#[derive(Debug)]
struct MemoryMetrics {
    total_bytes: u64,
    used_bytes: u64,
    available_bytes: u64,
    usage_percentage: f32,
}

#[derive(Debug)]
struct CpuMetrics {
    usage_percentage: f32,
    core_count: u32,
    load_average: LoadAverage,
}

#[derive(Debug)]
struct LoadAverage {
    one_minute: f32,
    five_minute: f32,
    fifteen_minute: f32,
}

#[derive(Debug)]
struct NetworkMetrics {
    bytes_sent: u64,
    bytes_received: u64,
    packets_sent: u64,
    packets_received: u64,
    active_connections: u32,
}

#[derive(Debug)]
enum MetricsCollectionError {
    SystemUnavailable,
    PermissionDenied,
    ParseError(String),
    Timeout,
}