# GitHub Copilot Instructions - Axum SSE Project

## Project Context
Real-time server metrics monitoring application using Rust backend (Axum) with embedded SvelteKit frontend.

## Current Feature: Server Status Page (002-the-user-interface)
Adding dedicated status page accessible via navigation for real-time server metrics visualization.

### Key Requirements
- Memory usage, CPU usage, uptime, and networking metrics display
- Real-time updates every 5 seconds via SSE
- Graphical representations (charts/graphs) for metrics
- Navigation bar integration
- Designed for competent computer users

### Technical Stack
- **Backend**: Rust 1.75+, Axum 0.7+, Tokio, Tower middleware
- **Metrics Collection**: sysinfo crate for system metrics
- **Frontend**: SvelteKit, TypeScript, Chart.js for visualizations
- **Real-time**: Server-Sent Events (SSE) for metric streaming
- **Deployment**: Single binary with embedded static assets

### API Endpoints
- `GET /api/server-status` - Snapshot of current metrics
- `GET /api/server-status-stream` - SSE stream for real-time updates
- `GET /status` - Frontend page route (SPA)

### Data Model
```rust
struct StatusPageData {
    server_metrics: ServerMetrics,
    collection_interval_seconds: u32,
    server_info: ServerInfo,
}

struct ServerMetrics {
    timestamp: DateTime<Utc>,
    memory_usage: MemoryMetrics,
    cpu_usage: CpuMetrics,
    uptime: Duration,
    network_metrics: NetworkMetrics,
}
```

### Constitutional Requirements
- TDD mandatory: Tests written before implementation
- Memory safety: Use Rust ownership system, no unsafe code
- Performance: <200ms response time, <50MB memory impact
- Single binary deployment with embedded frontend assets
- Structured logging with tracing

### Implementation Priorities
1. System metrics collection with sysinfo crate
2. API endpoints for status data and SSE streaming
3. Frontend SvelteKit page with Chart.js integration
4. Navigation bar updates
5. Contract tests for API validation

### Recent Changes
- Created comprehensive data model for server metrics
- Defined OpenAPI contracts for status endpoints
- Established SSE streaming pattern for real-time updates
- Planned Chart.js integration for metric visualization

## Development Guidelines
- Follow TDD: Write failing tests first, then implement
- Use async/await throughout for non-blocking operations
- Leverage existing SSE infrastructure patterns
- Maintain compatibility with current navigation structure
- Ensure embedded asset bundling for production deployment
