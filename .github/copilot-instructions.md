# axum-sse Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-09-20

## Active Technologies
- Rust + Axum (Backend)
- SvelteKit + TypeScript (Frontend)
- Server-Sent Events (SSE)
- Static Asset Embedding

## Project Structure
```
src/
├── main.rs                 # Application entry point
├── routes/                 # API and page route handlers
│   ├── api.rs             # SSE API endpoints
│   ├── pages.rs           # Static page serving
│   └── mod.rs
├── services/              # Core business logic
│   ├── sse_service.rs     # Server-Sent Events service
│   ├── static_service.rs  # Static asset serving
│   └── mod.rs
├── middleware/            # HTTP middleware
│   ├── security.rs        # CORS, headers, security
│   ├── logging.rs         # Request logging
│   └── mod.rs
├── models/                # Data models
│   ├── time_event.rs      # SSE time event structure
│   ├── connection_state.rs # Connection state management
│   └── mod.rs
└── lib.rs                 # Library exports

frontend/
├── src/
│   ├── app.html           # SvelteKit app template
│   ├── routes/            # SvelteKit pages
│   │   ├── +layout.svelte # Global layout with nav/banner
│   │   ├── +page.svelte   # Home page with SSE client
│   │   └── about/
│   │       └── +page.svelte # About page
│   └── lib/
│       └── stores/        # Svelte stores
│           ├── connection.ts # Connection state management
│           └── navigation.ts # Navigation state
├── build/                 # Production build output (embedded)
└── package.json

tests/                     # Integration and unit tests
```

## Commands

### Development
```bash
# Start development with frontend rebuild
BUILD_FRONTEND=1 cargo run

# Run backend only (serves embedded frontend)
cargo run

# Build frontend separately
cd frontend && npm run build

# Run tests
cargo test

# Build for production
cargo build --release
```

### Frontend Development
```bash
# Install dependencies
cd frontend && npm install

# Development server (standalone)
cd frontend && npm run dev

# Build production frontend
cd frontend && npm run build

# Preview production build
cd frontend && npm run preview
```

## Code Style
- Rust: Follow standard conventions with `rustfmt`
- TypeScript: ESLint + Prettier for SvelteKit
- CSS: Scoped component styles in Svelte files
- SSE: Custom event types with JSON payloads

## UI Code Generation Tasks

### Core UI Components

1. **Real-time Clock Display**
   - Large, prominent time display
   - UK format: DD/MM/YYYY HH:mm:ss
   - Visual connection state indicators
   - Last updated timestamp

2. **Connection Status Management**
   - Top banner with connection state
   - Color-coded status (green=connected, red=disconnected, yellow=connecting)
   - Auto-hide when stable connection
   - Exponential backoff reconnection

3. **Navigation Structure**
   - Header with app branding
   - Navigation links (Home, About)
   - Connection status indicator in nav
   - Responsive design

4. **SSE Client Implementation**
   - EventSource connection management
   - Custom event listener for 'time-update' events
   - Auto-reconnection with backoff
   - Connection state synchronization with stores

### Frontend Architecture

5. **Svelte Stores Pattern**
   - `connectionState`: Core connection data
   - `connectionStatus`: Derived connection UI state
   - `navigationState`: Route management
   - Reactive updates across components

6. **SvelteKit Integration**
   - Static adapter for embedded serving
   - SSR with client hydration
   - Asset optimization and hashing
   - Loading state management

7. **Responsive Design**
   - Mobile-first approach
   - Flexible layouts with CSS Grid/Flexbox
   - Accessible color contrast
   - Touch-friendly interactions

### Build Integration

8. **Asset Embedding**
   - `build.rs` script for frontend compilation
   - `include_dir!` macro for static assets
   - Conditional frontend builds
   - Production optimization

9. **Development Workflow**
   - Hot reload in development
   - Separate frontend/backend dev servers
   - Integrated production builds
   - Asset cache management

### UI Generation Prompts

When generating UI code for this project:

- **Use TypeScript** for all frontend logic
- **Implement SSE connections** with custom event types
- **Follow SvelteKit patterns** for routing and state management
- **Include connection state management** in all SSE-related components
- **Add proper error handling** for network failures
- **Implement responsive design** with mobile considerations
- **Use semantic HTML** for accessibility
- **Include loading states** for async operations
- **Add visual feedback** for connection status changes
- **Follow component composition** patterns in Svelte

### Example UI Patterns

```typescript
// SSE Connection Pattern
const eventSource = new EventSource('/api/time-stream');
eventSource.addEventListener('time-update', (event) => {
  const data = JSON.parse(event.data);
  // Update reactive state
});

// Store Integration Pattern
import { connectionState, connectionStatus } from '$lib/stores/connection';
$: isConnected = $connectionStatus.isConnected;

// Component Composition Pattern
<div class="status-banner" class:connected={isConnected}>
  {$connectionStatus.statusDisplay}
</div>
```

## Recent Changes
- 001-experimental-application-to: Added complete SSE-based real-time clock application
- Frontend: SvelteKit integration with TypeScript stores
- Backend: Axum server with embedded static asset serving
- SSE: Custom event broadcasting with connection management
- UI: Responsive design with connection status indicators

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->