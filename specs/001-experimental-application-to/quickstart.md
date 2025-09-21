# Quickstart: SSE Time Broadcasting Application

## Integration Test Scenarios

### Scenario 1: Complete User Journey

**Objective**: Verify end-to-end functionality from page load to time reception

**Steps**:
1. Start the application server
   ```bash
   cargo run
   ```

2. Open browser and navigate to `http://localhost:3000`

3. **Verify Initial Load**:
   - Dark theme interface loads
   - Navigation bar displays with app title and "About" link
   - Time display area is visible
   - No red connection banner visible

4. **Verify SSE Connection**:
   - Wait up to 11 seconds for first time event
   - Confirm time displays in UK format: DD/MM/YYYY HH:MM:SS
   - Verify automatic updates every 10 seconds

5. **Verify Navigation**:
   - Click "About" link in navigation
   - Confirm navigation to about page
   - Click back/home navigation
   - Verify time updates resume on main page

**Expected Results**:
- ✅ Page loads with dark theme
- ✅ SSE connection establishes automatically
- ✅ Time updates every 10 seconds in UK format
- ✅ Navigation works between home and about
- ✅ No errors in browser console

### Scenario 2: Connection Loss Recovery

**Objective**: Test red banner display and automatic reconnection

**Steps**:
1. Load application and verify normal operation
2. Simulate connection loss:
   - Stop the server (Ctrl+C) while page is open
   - Wait 15+ seconds for connection timeout
3. **Verify Error State**:
   - Red banner appears at top of page
   - Banner shows connection lost message
   - Time updates stop
4. Restart server:
   ```bash
   cargo run
   ```
5. **Verify Recovery**:
   - Red banner automatically disappears
   - SSE connection re-establishes
   - Time updates resume

**Expected Results**:
- ✅ Red banner appears on connection loss
- ✅ Banner disappears on reconnection
- ✅ Automatic recovery without user action
- ✅ Time updates resume after reconnection

### Scenario 3: Mobile Compatibility

**Objective**: Verify responsive design and mobile functionality

**Steps**:
1. Open browser developer tools
2. Enable mobile device simulation (iPhone/Android)
3. Navigate to `http://localhost:3000`
4. **Verify Mobile Layout**:
   - Interface adapts to mobile screen size
   - Navigation remains accessible
   - Time display is readable
   - SSE functionality works on mobile

**Expected Results**:
- ✅ Responsive design adapts to mobile screens
- ✅ Navigation works on touch devices
- ✅ SSE connection stable on mobile browsers
- ✅ Time display properly formatted and visible

### Scenario 4: Multiple Browser Tabs

**Objective**: Test concurrent SSE connections

**Steps**:
1. Open application in first browser tab
2. Open same URL in second browser tab
3. **Verify Both Tabs**:
   - Both tabs receive time updates
   - Updates are synchronized (same timestamps)
   - No interference between connections
4. Close one tab, verify other continues

**Expected Results**:
- ✅ Multiple SSE connections work simultaneously
- ✅ All tabs receive synchronized time updates
- ✅ Closing tabs doesn't affect others
- ✅ Server handles multiple connections efficiently

## Performance Validation

### Memory Usage Check
```bash
# Monitor memory usage while running
ps aux | grep axum-sse
# Should show <50MB memory usage
```

### Connection Capacity Test
```bash
# Use a tool like 'wrk' or 'ab' to test concurrent connections
# This is optional for experimental application
```

### Response Time Validation
```bash
# Test static page load times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3000/

# Test SSE endpoint establishment
curl -H "Accept: text/event-stream" http://localhost:3000/api/time/stream
```

## Troubleshooting Guide

### Common Issues

**SSE Connection Not Working**:
- Check browser console for CORS errors
- Verify Accept header is set to "text/event-stream"
- Confirm server is running on expected port

**Red Banner Stuck**:
- Check server logs for SSE streaming errors
- Verify EventSource reconnection logic
- Clear browser cache and reload

**Mobile Layout Issues**:
- Test with different viewport sizes
- Check CSS media queries are working
- Verify touch events are properly handled

**Time Format Wrong**:
- Check server timezone configuration
- Verify chrono crate formatting logic
- Test with different system locales

## Quick Development Commands

### Backend Development
```bash
# Run with auto-reload
cargo watch -x run

# Run tests
cargo test

# Check formatting
cargo fmt

# Check linting
cargo clippy
```

### Frontend Development
```bash
# Install dependencies
cd frontend && npm install

# Development mode (if running separately)
npm run dev

# Build for production
npm run build

# Type checking
npm run check
```

### Integration Commands
```bash
# Full clean build
cargo clean && npm run build && cargo build --release

# Run with release optimizations
cargo run --release

# Check single binary size
ls -lh target/release/axum-sse
```

## Validation Checklist

**Functional Requirements Validation**:
- [ ] FR-001: 10-second time broadcasts via SSE ✓
- [ ] FR-002: Real-time UI updates without user action ✓
- [ ] FR-003: Navigation bar with title and about link ✓
- [ ] FR-004: Modern dark theme interface ✓
- [ ] FR-005: Navigation between home and about ✓
- [ ] FR-006: Continuous updates regardless of navigation ✓
- [ ] FR-007: Red banner on connection loss/recovery ✓
- [ ] FR-008: HTTP access with no authentication ✓
- [ ] FR-009: UK date format (DD/MM/YYYY HH:MM:SS) ✓

**Performance Requirements**:
- [ ] Memory usage <50MB ✓
- [ ] Sub-millisecond static responses ✓
- [ ] 1000+ concurrent SSE connections ✓
- [ ] Mobile responsive design ✓

**Constitutional Compliance**:
- [ ] Single binary deployment ✓
- [ ] No external dependencies ✓
- [ ] Static assets embedded ✓
- [ ] Test-first development ✓

---

**Quickstart Status**: ✅ Complete
**Integration Scenarios**: 4 scenarios defined
**Ready for Task Generation**