# Data Model: Server Status Page

## Entities

### ServerMetrics
**Purpose**: Represents real-time system performance data

**Fields**:
- `timestamp: DateTime<Utc>` - When metrics were collected
- `memory_usage: MemoryMetrics` - RAM consumption data
- `cpu_usage: CpuMetrics` - Processor load data  
- `uptime: Duration` - Time since system/service start
- `network_metrics: NetworkMetrics` - Network activity data

**Validation Rules**:
- timestamp must be within last 10 seconds (freshness check)
- All metric values must be non-negative
- Memory percentages must be 0-100%
- CPU percentages must be 0-100% (can exceed 100% for multi-core)

**State Transitions**: N/A (immutable snapshots)

### MemoryMetrics
**Purpose**: Memory usage information

**Fields**:
- `total_bytes: u64` - Total system memory
- `used_bytes: u64` - Currently used memory
- `available_bytes: u64` - Available memory
- `usage_percentage: f32` - Used/Total as percentage

**Validation Rules**:
- used_bytes + available_bytes ≤ total_bytes
- usage_percentage = (used_bytes / total_bytes) * 100
- All byte values must be positive

### CpuMetrics  
**Purpose**: CPU utilization information

**Fields**:
- `usage_percentage: f32` - Current CPU usage (0-100+%)
- `core_count: u32` - Number of CPU cores
- `load_average: LoadAverage` - System load averages

**Validation Rules**:
- usage_percentage ≥ 0 (can exceed 100% for multi-core systems)
- core_count > 0
- Load averages must be non-negative

### LoadAverage
**Purpose**: System load average data

**Fields**:
- `one_minute: f32` - 1-minute load average
- `five_minute: f32` - 5-minute load average  
- `fifteen_minute: f32` - 15-minute load average

### NetworkMetrics
**Purpose**: Network activity statistics

**Fields**:
- `bytes_sent: u64` - Total bytes transmitted
- `bytes_received: u64` - Total bytes received
- `packets_sent: u64` - Total packets transmitted
- `packets_received: u64` - Total packets received
- `active_connections: u32` - Current active network connections

**Validation Rules**:
- All counters must be non-negative
- Counters are cumulative (monotonically increasing)

### StatusPageData
**Purpose**: Complete data structure for frontend consumption

**Fields**:
- `server_metrics: ServerMetrics` - Current system metrics
- `collection_interval_seconds: u32` - Update frequency (5 seconds)
- `server_info: ServerInfo` - Static server information

### ServerInfo
**Purpose**: Static server identification and configuration

**Fields**:
- `hostname: String` - Server hostname
- `version: String` - Application version
- `start_time: DateTime<Utc>` - When server started
- `environment: String` - Deployment environment (dev/staging/prod)

**Validation Rules**:
- hostname must be valid DNS hostname format
- version must follow semantic versioning
- start_time must be in the past

## Relationships

- `StatusPageData` contains one `ServerMetrics`
- `ServerMetrics` contains one each of `MemoryMetrics`, `CpuMetrics`, `NetworkMetrics`
- `CpuMetrics` contains one `LoadAverage`
- `StatusPageData` contains one `ServerInfo`

## Error States

### MetricsCollectionError
**Purpose**: Represents failures in system metrics gathering

**Variants**:
- `SystemUnavailable` - Cannot access system information
- `PermissionDenied` - Insufficient privileges for metrics
- `ParseError(String)` - Failed to parse system data
- `Timeout` - Metrics collection exceeded time limit

### MetricsResponse
**Purpose**: Result type for metrics API responses

**Variants**:
- `Ok(StatusPageData)` - Successful metrics collection
- `PartialData(StatusPageData, Vec<MetricsCollectionError>)` - Some metrics unavailable
- `Error(MetricsCollectionError)` - Complete failure

## Data Flow

1. **Collection**: System metrics gathered every 1 second, cached
2. **Aggregation**: Latest cached data packaged into `StatusPageData`
3. **Transmission**: Data sent via SSE every 5 seconds to connected clients
4. **Display**: Frontend receives and visualizes data in real-time

## Performance Considerations

- Metrics collection runs in background task, not blocking requests
- In-memory caching prevents redundant system calls
- Structured data enables efficient JSON serialization
- Immutable data structures prevent concurrent modification issues