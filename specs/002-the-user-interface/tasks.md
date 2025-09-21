# Tasks: Server Status Page

**Input**: Design documents from `/specs/002-the-user-interface/`

**Prerequisites**: plan.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

## Execution Flow (main)

```text
1. Load plan.md from feature directory
   → ✅ Implementation plan found
   → ✅ Tech stack: Rust 1.75+, Axum 0.7+, SvelteKit, TypeScript 5.0+
   → ✅ Libraries: sysinfo, Chart.js, tokio, tower
2. Load optional design documents:
   → ✅ data-model.md: ServerMetrics, MemoryMetrics, CpuMetrics, NetworkMetrics entities
   → ✅ contracts/: api.yaml OpenAPI spec + contract_tests.rs TDD setup
   → ✅ research.md: sysinfo crate, SSE streaming, Chart.js visualization
3. Generate tasks by category:
   → ✅ Setup: sysinfo dependency, frontend Chart.js library
   → ✅ Tests: contract tests for /api/server-status and /api/server-status-stream
   → ✅ Core: metrics models, collection service, API endpoints
   → ✅ Integration: SSE streaming, frontend status page, navigation
   → ✅ Polish: error handling, performance validation, visual improvements
4. Apply task rules:
   → ✅ Different files = [P] parallel execution
   → ✅ Same file = sequential dependencies
   → ✅ Tests before implementation (TDD)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency graph
7. Create parallel execution examples
8. Validate task completeness:
   → ✅ API contracts have failing tests
   → ✅ All entities have Rust models
   → ✅ Frontend status page with Chart.js
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions

- **Backend**: `src/` at repository root (Rust/Axum)
- **Frontend**: `frontend/src/` (SvelteKit/TypeScript)
- **Tests**: `tests/` for integration, `src/` for unit tests

## Phase 3.1: Setup

- [x] T001 Add sysinfo dependency to Cargo.toml for system metrics collection
- [x] T002 [P] Add Chart.js dependency to frontend/package.json for metric visualization
- [x] T003 [P] Configure cargo clippy rules for metrics module in clippy.toml

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3

### Critical: These tests MUST be written and MUST FAIL before ANY implementation

- [x] T004 [P] Contract test GET /api/server-status endpoint in tests/contract/server_status_get.rs
- [x] T005 [P] Contract test GET /api/server-status-stream SSE endpoint in tests/contract/server_status_stream.rs
- [x] T006 [P] Integration test metrics collection service in tests/integration/metrics_collection.rs
- [x] T007 [P] Integration test SSE streaming with multiple clients in tests/integration/sse_streaming.rs
- [x] T008 [P] Unit test for ServerMetrics validation in src/models/server_metrics.rs (test module)

## Phase 3.3: Core Implementation (ONLY after tests are failing)

- [x] T009 [P] ServerMetrics model with validation in src/models/server_metrics.rs [COMPLETED]
- [x] T010 [P] MemoryMetrics model in src/models/memory_metrics.rs
- [x] T011 [P] CpuMetrics and LoadAverage models in src/models/cpu_metrics.rs
- [x] T012 [P] NetworkMetrics model in src/models/network_metrics.rs
- [x] T013 [P] StatusPageData wrapper model in src/models/status_page_data.rs
- [x] T014: MetricsCollectionError enum in src/models/metrics_errors.rs [COMPLETED]
- [x] T015: MetricsService implementation in src/services/metrics_service.rs [COMPLETED]
- [x] T016: MetricsCache implementation in src/services/metrics_cache.rs [COMPLETED]
- [x] T017: Server status API endpoint in src/routes/server_status.rs [COMPLETED]
- [x] T018: Server status SSE endpoint in src/routes/server_status_stream.rs [COMPLETED]
- [x] T019: Register server status routes in main.rs [COMPLETED]

## Phase 3.4: Frontend Implementation

- [ ] T020 [P] Server status page component in frontend/src/routes/status/+page.svelte
- [ ] T021 [P] Chart.js integration service in frontend/src/lib/charts.ts
- [ ] T022 [P] SSE client service for real-time updates in frontend/src/lib/sse-client.ts
- [ ] T023 [P] Metrics display components in frontend/src/lib/components/MetricsDisplay.svelte
- [ ] T024 [P] Memory usage chart component in frontend/src/lib/components/MemoryChart.svelte
- [ ] T025 [P] CPU usage chart component in frontend/src/lib/components/CpuChart.svelte
- [ ] T026 [P] Network metrics visualization in frontend/src/lib/components/NetworkMetrics.svelte
- [ ] T027 Update navigation bar with Status link in frontend/src/lib/components/Navigation.svelte

## Phase 3.5: Integration & Error Handling

- [ ] T028 Error handling for metrics collection failures in src/services/metrics_service.rs
- [ ] T029 SSE connection management and reconnection in frontend/src/lib/sse-client.ts
- [ ] T030 Graceful degradation for unavailable metrics in frontend/src/routes/status/+page.svelte
- [ ] T031 Performance monitoring for metrics collection overhead in src/services/metrics_cache.rs

## Phase 3.6: Polish & Validation

- [ ] T032 [P] Unit tests for metrics models validation in src/models/ (test modules)
- [ ] T033 [P] Frontend component tests in frontend/src/lib/components/ (test files)
- [ ] T034 [P] Performance validation: <200ms API response time
- [ ] T035 [P] Memory usage validation: <50MB additional overhead
- [ ] T036 [P] SSE connection scaling test: 1000+ concurrent connections
- [ ] T037 [P] Visual improvements and responsive design in frontend styles
- [ ] T038 [P] Update project documentation for status page feature
- [ ] T039 Run quickstart.md validation scenarios for acceptance testing

## Dependencies

**Critical Path:**

- Setup (T001-T003) → Tests (T004-T008) → Core Models (T009-T014) → Services (T015-T016) → API Routes (T017-T019) → Frontend (T020-T027) → Integration (T028-T031) → Polish (T032-T039)

**Within Phases:**

- T015 (MetricsService) blocks T016 (caching), T017 (API endpoint)
- T016 (caching) blocks T018 (SSE streaming)
- T017-T018 (API endpoints) block T019 (route registration)
- T020 (status page) blocks T022-T027 (frontend components)
- T021 (Chart.js service) blocks T024-T026 (chart components)

## Parallel Execution Examples

### Phase 3.2 - All Contract Tests (Parallel)

```bash
# Launch T004-T008 together:
cargo test --test contract_server_status_get &
cargo test --test contract_server_status_stream &
cargo test --test integration_metrics_collection &
cargo test --test integration_sse_streaming &
cargo test src::models::server_metrics::tests &
wait
```

### Phase 3.3 - Model Creation (Parallel T009-T014)

```bash
# All models can be created in parallel (different files):
touch src/models/server_metrics.rs &
touch src/models/memory_metrics.rs &
touch src/models/cpu_metrics.rs &
touch src/models/network_metrics.rs &
touch src/models/status_page_data.rs &
touch src/models/metrics_errors.rs &
wait
```

### Phase 3.4 - Frontend Components (Parallel T020-T027)

```bash
# Frontend components in parallel (different files):
mkdir -p frontend/src/routes/status &
touch frontend/src/routes/status/+page.svelte &
touch frontend/src/lib/charts.ts &
touch frontend/src/lib/sse-client.ts &
touch frontend/src/lib/components/MetricsDisplay.svelte &
touch frontend/src/lib/components/MemoryChart.svelte &
touch frontend/src/lib/components/CpuChart.svelte &
touch frontend/src/lib/components/NetworkMetrics.svelte &
wait
```

## File Structure After Completion

```text
src/
├── models/
│   ├── server_metrics.rs      (T009)
│   ├── memory_metrics.rs      (T010)
│   ├── cpu_metrics.rs         (T011)
│   ├── network_metrics.rs     (T012)
│   ├── status_page_data.rs    (T013)
│   └── metrics_errors.rs      (T014)
├── services/
│   ├── metrics_service.rs     (T015)
│   └── metrics_cache.rs       (T016)
└── routes/
    ├── server_status.rs       (T017)
    └── server_status_stream.rs (T018)

frontend/src/
├── routes/status/
│   └── +page.svelte           (T020)
├── lib/
│   ├── charts.ts              (T021)
│   ├── sse-client.ts          (T022)
│   └── components/
│       ├── MetricsDisplay.svelte    (T023)
│       ├── MemoryChart.svelte       (T024)
│       ├── CpuChart.svelte          (T025)
│       ├── NetworkMetrics.svelte    (T026)
│       └── Navigation.svelte        (T027, updated)

tests/
├── contract/
│   ├── server_status_get.rs         (T004)
│   └── server_status_stream.rs      (T005)
└── integration/
    ├── metrics_collection.rs        (T006)
    └── sse_streaming.rs             (T007)
```

## Notes

- **[P] tasks**: Different files, can run in parallel safely
- **Sequential tasks**: Same file or dependencies, must run in order
- **TDD Requirement**: All tests (T004-T008) MUST FAIL before implementing (T009+)
- **Performance targets**: <200ms API response, <50MB memory, 1000+ SSE connections
- **Constitutional compliance**: Memory safety (Rust ownership), async/await throughout
- **Commit strategy**: Commit after each completed task for granular history

## Validation Checklist

- [ ] ✅ All API contracts have failing tests (T004-T005)
- [ ] ✅ All entities have Rust models (T009-T014)
- [ ] ✅ SSE streaming implementation (T018, T022)
- [ ] ✅ Chart.js visualization (T021, T024-T026)
- [ ] ✅ Navigation integration (T027)
- [ ] ✅ Error handling and graceful degradation (T028-T031)
- [ ] ✅ Performance validation (T034-T036)
- [ ] ✅ User acceptance testing (T039)
