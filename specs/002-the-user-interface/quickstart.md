# Quickstart: Server Status Page

## Overview
This quickstart validates the server status page feature by walking through all user scenarios and verifying the implementation meets requirements.

## Prerequisites
- Server running on http://localhost:3000
- Modern web browser with JavaScript enabled
- Developer tools access for debugging

## Test Scenarios

### Scenario 1: Navigation Access
**Goal**: Verify status page is accessible via navigation bar

**Steps**:
1. Open browser to http://localhost:3000
2. Locate navigation bar at top of page
3. Look for "Status" or "Server Status" link/button
4. Click the status navigation link

**Expected Results**:
- Status link should be clearly visible in navigation bar
- Click should navigate to `/status` URL
- Page should load without errors
- Status page should display server metrics

**Validation Commands**:
```bash
# Verify status page route returns 200
curl -i http://localhost:3000/status

# Verify API endpoint returns JSON
curl -H "Accept: application/json" http://localhost:3000/api/server-status
```

### Scenario 2: Metrics Display
**Goal**: Verify all required metrics are displayed with graphical representations

**Steps**:
1. Navigate to status page (from Scenario 1)
2. Observe the displayed metrics sections
3. Verify presence of memory usage display
4. Verify presence of CPU usage display  
5. Verify presence of uptime display
6. Verify presence of network metrics display
7. Check for graphical elements (charts, graphs, progress bars)

**Expected Results**:
- Memory usage shown with percentage and bytes (total/used/available)
- CPU usage shown with percentage and core count
- Uptime displayed in human-readable format
- Network metrics showing bytes/packets sent/received and connections
- At least some metrics displayed as graphs or visual indicators
- All values should be reasonable/realistic

**Validation**:
- All metrics should have numerical values > 0
- Memory percentage should be 0-100%
- CPU percentage should be ≥ 0 (can exceed 100% for multi-core)
- Uptime should increase over time

### Scenario 3: Real-time Updates
**Goal**: Verify metrics update automatically every 5 seconds

**Steps**:
1. Navigate to status page
2. Note the current values of displayed metrics
3. Wait 5 seconds without refreshing page
4. Observe if metrics have updated
5. Wait another 5 seconds
6. Verify consistent update intervals

**Expected Results**:
- Metrics should change/update without manual page refresh
- Updates should occur approximately every 5 seconds
- CPU and memory values should reflect current system state
- Network counters should increase (if system has activity)
- Uptime should continuously increase

**Validation Commands**:
```bash
# Test SSE endpoint directly
curl -N http://localhost:3000/api/server-status-stream

# Verify events arrive every ~5 seconds
# Look for "event: status-update" lines
```

### Scenario 4: Page Navigation Consistency
**Goal**: Verify navigation patterns are consistent with existing application

**Steps**:
1. Navigate to main page (/)
2. Use status page navigation link
3. Navigate back to main page using existing navigation
4. Return to status page via navigation
5. Test direct URL access to /status

**Expected Results**:
- Navigation should work bidirectionally
- Status page navigation should match existing UI patterns
- Direct URL access to /status should work
- Browser back/forward buttons should work correctly
- Navigation state should be consistent

### Scenario 5: Error Handling
**Goal**: Verify graceful handling of error conditions

**Steps**:
1. Open browser developer tools (Network tab)
2. Navigate to status page
3. Observe network requests to `/api/server-status-stream`
4. Simulate network interruption (disable network temporarily)
5. Re-enable network connection
6. Verify page behavior during connectivity issues

**Expected Results**:
- Page should not crash during network issues
- Error states should be handled gracefully
- Connection should resume when network is restored
- User should receive appropriate feedback about connection status

## Performance Validation

### Response Time Check
```bash
# API endpoint should respond quickly
time curl http://localhost:3000/api/server-status

# Response should be < 200ms
```

### Memory Usage Check
```bash
# Monitor server memory usage while status page is active
ps aux | grep axum-sse

# Memory usage should remain stable with multiple clients
```

### Concurrent Connections Test
```bash
# Test multiple SSE connections
for i in {1..10}; do
  curl -N http://localhost:3000/api/server-status-stream &
done

# Server should handle multiple concurrent connections
# All connections should receive updates
```

## Acceptance Criteria Verification

**FR-001** ✓ Dedicated server status page accessible via URL routing
- Verify: `/status` URL loads successfully

**FR-002** ✓ Navigation bar includes clearly labeled link to status page  
- Verify: Status link visible and functional in navigation

**FR-003** ✓ Page displays memory, CPU, uptime, and networking metrics
- Verify: All four metric categories present and populated

**FR-004** ✓ Metrics update automatically every 5 seconds
- Verify: Observe automatic updates at 5-second intervals

**FR-005** ✓ Metrics presented with graphical representations
- Verify: At least some metrics shown as charts/graphs/visual indicators

**FR-006** ✓ Designed for competent computer users
- Verify: Technical metrics displayed with appropriate detail

**FR-007** ✓ Handles cases where status information is unavailable
- Verify: Graceful error handling and user feedback

**FR-008** ✓ Navigation consistent with existing application patterns
- Verify: Status page navigation matches existing UI patterns

## Success Criteria
- [ ] All test scenarios pass
- [ ] All functional requirements verified
- [ ] Performance targets met
- [ ] Error handling works correctly
- [ ] User experience is smooth and consistent

## Troubleshooting

**Status page not loading**:
- Check server logs for errors
- Verify `/status` route is registered
- Check SPA fallback routing configuration

**Metrics not updating**:
- Verify SSE endpoint `/api/server-status-stream` is accessible
- Check browser developer tools for SSE connection errors
- Confirm metrics collection is working on server side

**Navigation not working**:
- Verify navigation component includes status page link
- Check SvelteKit routing configuration
- Confirm link URLs are correct

**Charts not displaying**:
- Check browser console for JavaScript errors
- Verify Chart.js library is loaded
- Confirm chart data format is correct