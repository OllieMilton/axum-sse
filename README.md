# Axum SSE - Real-time Server Metrics Monitor

A real-time server monitoring application built with Rust backend (Axum) and embedded SvelteKit frontend, featuring Server-Sent Events (SSE) for live metrics streaming.

## Features

- **Real-time Metrics Monitoring**: Live server metrics with 5-second updates via SSE
- **System Metrics Collection**: Memory usage, CPU utilization, uptime, and network activity
- **Interactive Visualizations**: Chart.js powered graphs and charts
- **Single Binary Deployment**: Embedded frontend assets for easy deployment
- **Modern Web UI**: SvelteKit frontend with responsive design
- **RESTful API**: JSON API endpoints for metrics access

## Quick Start

### Prerequisites

- Rust 1.75+ with Cargo
- Node.js 18+ and npm (for frontend development)

### Running the Application

1. **Clone and build**:

   ```bash
   git clone <repository-url>
   cd axum-sse
   cargo run
   ```

2. **Access the application**:

   - Main page: <http://localhost:3000>
   - Server status: <http://localhost:3000/status>
   - API endpoint: <http://localhost:3000/api/server-status>

The build process automatically compiles and embeds the SvelteKit frontend into the Rust binary.

## Architecture

### Backend (Rust + Axum)

- **Framework**: Axum 0.7 with Tokio async runtime
- **Metrics Collection**: sysinfo crate + /proc filesystem parsing
- **Real-time Updates**: Server-Sent Events (SSE) streaming
- **API**: RESTful JSON endpoints with OpenAPI contracts

### Frontend (SvelteKit + TypeScript)

- **Framework**: SvelteKit with TypeScript
- **Visualizations**: Chart.js for real-time charts
- **Real-time**: EventSource API for SSE consumption
- **Styling**: Modern responsive CSS

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Main application page |
| `/status` | GET | Server status monitoring page |
| `/api/server-status` | GET | Current server metrics snapshot |
| `/api/server-status-stream` | GET | SSE stream for real-time metrics |

## Metrics Collected

- **Memory Usage**: Total, used, available memory in bytes and percentages
- **CPU Usage**: System and user CPU utilization percentages
- **Uptime**: Server runtime duration
- **Network Activity**: Bytes received/transmitted per second by interface

## Development

### Project Structure

```text
axum-sse/
├── src/                 # Rust backend source
│   ├── handlers/        # HTTP request handlers
│   ├── models/          # Data models and serialization
│   ├── services/        # Business logic (metrics collection)
│   └── main.rs         # Application entry point
├── frontend/           # SvelteKit frontend source
│   ├── src/           # Svelte components and pages
│   └── static/        # Static assets
├── tests/             # Integration tests
├── specs/             # Feature specifications
└── build.rs          # Build script for frontend embedding
```

### Building for Production

```bash
# Build optimized binary with embedded frontend
cargo build --release

# Run production binary
./target/release/axum-sse
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*'

# Test API endpoints
curl http://localhost:3000/api/server-status
curl http://localhost:3000/api/server-status-stream
```

## Configuration

The application uses sensible defaults:

- **Port**: 3000 (configurable via environment)
- **Metrics Interval**: 5 seconds
- **Frontend Build**: Always enabled in build.rs

## Contributing

This project follows Test-Driven Development (TDD) practices:

1. Write failing tests first
2. Implement minimal code to pass tests
3. Refactor while maintaining test coverage

## License

[Add your license here]
