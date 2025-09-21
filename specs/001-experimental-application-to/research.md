# Research: SSE Time Broadcasting Application

## Technical Decisions and Research Findings

### 1. Backend Framework Choice

**Decision**: Axum 0.7+ with Tokio runtime

**Rationale**: 
- Constitutional requirement for Rust 1.75+ and Axum 0.7+
- Excellent SSE support with built-in streaming capabilities
- High performance async runtime with Tokio
- Strong ecosystem integration with Tower middleware
- Type-safe routing and handler system

**Alternatives Considered**:
- Warp: Less mature SSE support, more complex middleware system
- Actix-web: Good performance but moving away from actor model
- Rocket: Simpler API but less async-first design

### 2. Frontend Framework Choice

**Decision**: SvelteKit with TypeScript

**Rationale**:
- Excellent build tooling for static asset generation
- Small bundle sizes suitable for embedding
- Built-in SSE/EventSource support
- Mobile-responsive capabilities out of the box
- TypeScript support for type safety

**Alternatives Considered**:
- React/Next.js: Larger bundle sizes, more complex for simple use case
- Vue/Nuxt: Good option but SvelteKit has better static generation
- Vanilla JS: Would require custom build tooling

### 3. Asset Embedding Strategy

**Decision**: `include_dir!` macro with compile-time embedding

**Rationale**:
- Constitutional requirement for static asset embedding
- Zero external dependencies in production
- Single binary deployment
- Rust's `include_dir` crate provides efficient asset serving

**Alternatives Considered**:
- Runtime file serving: Violates constitutional single binary requirement
- Base64 embedding: Less efficient for larger assets
- External CDN: Adds external dependencies

### 4. SSE Implementation Pattern

**Decision**: Axum streaming response with tokio::time::interval

**Rationale**:
- Native Axum SSE support via streaming responses
- Tokio interval provides reliable 10-second timing
- Built-in backpressure handling for connection management
- Efficient memory usage with streaming

**Alternatives Considered**:
- WebSockets: More complex than needed for one-way communication
- Polling: Inefficient and not real-time
- Custom streaming: Reinventing existing Axum capabilities

### 5. Time Formatting Approach

**Decision**: chrono crate with UK locale formatting

**Rationale**:
- Comprehensive date/time handling library
- Built-in formatting for DD/MM/YYYY HH:MM:SS pattern
- Timezone handling capabilities if needed later
- Well-maintained and widely used

**Alternatives Considered**:
- std::time: Limited formatting capabilities
- time crate: Good alternative but chrono more established
- Custom formatting: Error-prone and unnecessary

### 6. Mobile Compatibility Strategy

**Decision**: CSS Grid/Flexbox with responsive design principles

**Rationale**:
- Modern CSS provides excellent mobile support
- SvelteKit's built-in responsive capabilities
- No need for separate mobile frameworks
- Progressive enhancement approach

**Alternatives Considered**:
- Separate mobile app: Violates single binary principle
- Mobile-first frameworks: Unnecessary complexity
- Native mobile: Outside project scope

### 7. Connection State Management

**Decision**: EventSource error handling with UI state management

**Rationale**:
- Browser EventSource API provides built-in reconnection
- Simple state management for connection status
- Red banner UI pattern for error visibility
- Automatic recovery without user intervention

**Alternatives Considered**:
- Manual WebSocket management: More complex implementation
- Polling fallback: Degrades user experience
- No error handling: Poor user experience

## Build Integration Strategy

### Frontend Build Process
1. SvelteKit builds to `frontend/build/` directory
2. Rust build.rs script copies assets to `src/static_assets/`
3. `include_dir!` macro embeds assets at compile time
4. Axum serves embedded assets through routing

### Development Workflow
1. Frontend development with `npm run dev` (hot reload)
2. Backend development with `cargo run` (auto-reload with cargo-watch)
3. Integration testing with full build pipeline
4. Production build creates single binary with embedded assets

## Performance Considerations

### Memory Usage
- Target: <50MB total memory usage
- SvelteKit builds typically <1MB gzipped
- Rust binary overhead ~10-20MB
- SSE connection memory ~1KB per connection

### Concurrent Connections
- Target: 1000+ concurrent SSE connections
- Tokio async runtime handles connection efficiently
- Memory usage scales linearly with connections
- Connection cleanup on client disconnect

### Response Times
- Static asset serving: <1ms (in-memory)
- SSE endpoint establishment: <1ms
- Time broadcast latency: <10ms after interval

## Security Considerations

### No Authentication Required
- Constitutional requirement for HTTP with no auth
- Experimental application scope justifies this approach
- CORS configuration for browser security
- No sensitive data transmission

### Input Validation
- No user inputs to validate (time-only application)
- Static asset serving through Axum's secure routing
- Standard HTTP security headers via Tower middleware

## Deployment Strategy

### Single Binary Deployment
- Constitutional requirement achieved
- All assets embedded at compile time
- No external file dependencies
- Cross-platform compilation support

### Configuration
- Port configuration via environment variables
- Optional timezone configuration for time display
- Logging level configuration for debugging

---

**Research Status**: ✅ Complete
**No NEEDS CLARIFICATION items remaining**
**Ready for Phase 1: Design & Contracts**