# Data Model: SSE Time Broadcasting Application

## Core Entities

### Time Event

**Purpose**: Represents a timestamp broadcast event sent via Server-Sent Events

**Fields**:
- `timestamp`: DateTime in UTC
- `formatted_time`: String in UK format (DD/MM/YYYY HH:MM:SS)
- `event_id`: Unique identifier for SSE event ordering

**Validation Rules**:
- Timestamp must be current system time (±1 second tolerance)
- Formatted time must match UK date pattern: `\d{2}/\d{2}/\d{4} \d{2}:\d{2}:\d{2}`
- Event ID must be monotonically increasing

**State Transitions**:
1. **Created** → Generated every 10 seconds by timer
2. **Broadcasted** → Sent to all active SSE connections
3. **Delivered** → Received by client browsers

**Serialization Format** (JSON):
```json
{
  "timestamp": "2025-09-20T14:30:00Z",
  "formatted_time": "20/09/2025 14:30:00",
  "event_id": 12345
}
```

### Connection State

**Purpose**: Tracks the status of client SSE connections for UI feedback

**Fields**:
- `status`: Enum (Connected, Disconnected, Reconnecting)
- `last_event_received`: Optional timestamp of last successful event
- `connection_start`: Timestamp when connection was established
- `error_message`: Optional error description

**Validation Rules**:
- Status must be one of defined enum values
- Last event received must be within 15 seconds for Connected status
- Connection start must be valid timestamp
- Error message only present when status is Disconnected

**State Transitions**:
1. **Disconnected** (initial state)
2. **Reconnecting** → When attempting to establish connection
3. **Connected** → Successfully receiving events
4. **Disconnected** → Connection lost or error occurred

**Client-Side State**:
```typescript
interface ConnectionState {
  status: 'connected' | 'disconnected' | 'reconnecting';
  lastEventReceived?: Date;
  connectionStart: Date;
  errorMessage?: string;
}
```

### Navigation State

**Purpose**: Tracks user's current page location for maintaining SSE connection

**Fields**:
- `current_page`: Enum (Home, About)
- `sse_active`: Boolean indicating if SSE should be connected
- `navigation_history`: Array of previous page visits

**Validation Rules**:
- Current page must be one of defined routes
- SSE active only when on Home page
- Navigation history limited to last 10 entries

**State Transitions**:
1. **Home** (default) → SSE connection active, time display visible
2. **About** → SSE connection maintained, time display hidden
3. **Home** → Return to time display, SSE continues

**Client-Side State**:
```typescript
interface NavigationState {
  currentPage: '/' | '/about';
  sseActive: boolean;
  navigationHistory: string[];
}
```

## Data Flow

### SSE Broadcast Flow
```
Timer (10s) → TimeEvent Creation → JSON Serialization → SSE Broadcast → Client Reception → UI Update
```

### Connection Management Flow
```
Page Load → EventSource Creation → Connection State Update → Event Reception → State Validation
```

### Navigation Flow
```
User Click → Route Change → Navigation State Update → SSE State Management → UI Rendering
```

## Persistence Strategy

**No Persistent Storage Required**: Application is stateless and experimental

**In-Memory State**:
- Connection tracking for active SSE clients
- Current navigation state per client session
- Last event ID for proper SSE event ordering

**State Management**:
- Backend: Minimal connection tracking in Axum handler state
- Frontend: SvelteKit stores for reactive state management
- No database or file storage needed

## Validation Patterns

### Time Event Validation
```rust
impl TimeEvent {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            timestamp: now,
            formatted_time: format_uk_time(now),
            event_id: next_event_id(),
        }
    }
    
    fn is_valid(&self) -> bool {
        // Validation logic
    }
}
```

### Connection State Validation
```typescript
function validateConnectionState(state: ConnectionState): boolean {
    if (state.status === 'connected') {
        return state.lastEventReceived && 
               (Date.now() - state.lastEventReceived.getTime()) < 15000;
    }
    return true;
}
```

## Error Handling Patterns

### Backend Error States
- **Timer Failure**: Restart timer mechanism, log error
- **Serialization Error**: Skip event, continue with next
- **Connection Broadcast Error**: Remove failed connections

### Frontend Error States
- **Connection Failed**: Show red banner, attempt reconnection
- **Invalid Data Received**: Log error, continue with previous data
- **Navigation Error**: Reset to home page, maintain SSE connection

---

**Data Model Status**: ✅ Complete  
**Entities Defined**: 3 (Time Event, Connection State, Navigation State)  
**Ready for Contract Generation**