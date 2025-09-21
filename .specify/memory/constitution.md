<!--
Sync Impact Report:
- Version change: template → 1.0.0
- Added sections: All core principles defined for Rust backend + SPA
- Principles created: Memory Safety, API-First Design, Test-First Development, Static Asset Embedding, Performance by Default
- Templates requiring updates: ✅ updated for Rust/Axum development
- Follow-up TODOs: None
-->

# Axum SSE Constitution

## Core Principles

### I. Memory Safety First

All code MUST leverage Rust's ownership system and compile-time guarantees; Unsafe code blocks require explicit justification and peer review; Zero-copy operations preferred for performance-critical paths; Resource cleanup must be automatic via RAII patterns.

### II. API-First Design

Every feature starts with API contract definition; OpenAPI/JSON schemas define all endpoints before implementation; Server-Sent Events (SSE) endpoints must follow consistent event stream patterns; API versioning required for breaking changes using semantic versioning.

### III. Test-First Development (NON-NEGOTIABLE)

TDD mandatory: Tests written → Tests fail → Then implement; Unit tests for all business logic using `cargo test`; Integration tests for all HTTP endpoints and SSE streams; Property-based testing with `proptest` for complex data structures.

### IV. Static Asset Embedding

Single page application assets MUST be embedded at compile time; Use `include_dir!` or similar macros for frontend asset bundling; No external file dependencies in production deployments; Asset routing integrated with Axum's routing system.

### V. Performance by Default

Sub-millisecond response times for non-streaming endpoints; SSE connections must handle 1000+ concurrent clients; Memory usage under 50MB for basic deployments; Async/await throughout, no blocking operations in request handlers.

## Technical Constraints

**Required Stack**: Rust 1.75+, Axum 0.7+, Tokio runtime, Tower middleware
**Frontend Integration**: Static assets embedded via build process, served through Axum routes
**Deployment**: Single binary with no external dependencies
**Monitoring**: Structured logging with `tracing`, metrics via middleware
**Security**: CORS configuration required, input validation on all endpoints

## Development Workflow

**Feature Development**: All features follow `/specify` → `/plan` → `/tasks` → `/implement` workflow
**Code Reviews**: Two-person approval required, clippy warnings must be resolved
**Testing Gates**: All tests pass, coverage >80% for new code, no unsafe code without justification
**Release Process**: Semantic versioning, changelog generation, single binary deployment

## Governance

This constitution supersedes all other development practices; All PRs and code reviews must verify constitutional compliance; Complexity deviations must be documented with justification; Breaking changes require constitutional amendment process; Use agent-specific guidance files (.github/copilot-instructions.md, CLAUDE.md, etc.) for runtime development guidance.

**Version**: 1.0.0 | **Ratified**: 2025-09-20 | **Last Amended**: 2025-09-20
