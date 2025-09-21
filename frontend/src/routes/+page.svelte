<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { connectionState, connectionStatus } from '$lib/stores/connection';
	import { dateTimeStore, isStoredTimeStale, getRelativeTimeString, shouldShowStaleWarning } from '$lib/stores/datetime';
	
	// Local state for SSE connection management
	let eventSource: EventSource | null = null;
	let connectionAttempts = 0;
	let isManuallyDisconnected = false;
	
	// Reactive statements for connection
	$: isConnected = $connectionStatus.isConnected;
	$: statusDisplay = $connectionStatus.statusDisplay;
	$: reconnectDelay = $connectionStatus.reconnectionDelayMs;
	
	// Reactive statements for datetime (from persistent store)
	$: currentTime = $dateTimeStore.currentTime || 'No time received yet';
	$: lastUpdated = $dateTimeStore.lastUpdated;
	$: isStale = shouldShowStaleWarning($dateTimeStore);
	$: relativeTime = getRelativeTimeString(lastUpdated);
	
	// Connect to SSE stream
	function connectToTimeStream() {
		if (eventSource) {
			eventSource.close();
		}
		
		connectionAttempts++;
		isManuallyDisconnected = false;
		
		// Set connecting state
		connectionState.startConnecting();
		
		console.log(`Connecting to SSE stream (attempt ${connectionAttempts})...`);
		
		try {
			eventSource = new EventSource('/api/time-stream');
			
			eventSource.onopen = (event) => {
				console.log('SSE connection opened:', event);
				connectionState.connect(`sse-${Date.now()}`);
				connectionAttempts = 0;
			};
			
			// Listen for the specific 'time-update' event type
			eventSource.addEventListener('time-update', (event) => {
				try {
					const data = JSON.parse(event.data);
					// Use the enhanced connection method that updates both connection and datetime
					connectionState.pingWithTime(data.formatted_time);
					console.log('Received time update:', data);
				} catch (error) {
					console.error('Failed to parse SSE data:', error);
				}
			});
			
			eventSource.onerror = (error) => {
				console.error('SSE connection error:', error);
				connectionState.disconnect();
				
				if (!isManuallyDisconnected && eventSource?.readyState !== EventSource.CONNECTING) {
					// Auto-reconnect with exponential backoff
					const delay = Math.min(1000 * Math.pow(2, connectionAttempts - 1), 30000);
					console.log(`Reconnecting in ${delay}ms...`);
					setTimeout(connectToTimeStream, delay);
				}
			};
			
		} catch (error) {
			console.error('Failed to create SSE connection:', error);
			connectionState.disconnect();
		}
	}
	
	// Disconnect from SSE stream
	function disconnect() {
		isManuallyDisconnected = true;
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}
		connectionState.disconnect();
	}
	
	// Clear stored time
	function clearStoredTime() {
		dateTimeStore.clearTime();
	}
	
	// Manual time broadcast test
	async function manualBroadcast() {
		try {
			const response = await fetch('/api/broadcast', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					message: 'Manual broadcast test'
				})
			});
			
			if (response.ok) {
				const result = await response.json();
				console.log('Manual broadcast sent:', result);
			} else {
				console.error('Manual broadcast failed:', response.statusText);
			}
		} catch (error) {
			console.error('Manual broadcast error:', error);
		}
	}
	
	// Lifecycle
	onMount(() => {
		connectToTimeStream();
	});
	
	onDestroy(() => {
		isManuallyDisconnected = true;
		if (eventSource) {
			eventSource.close();
		}
	});
</script>

<svelte:head>
	<title>Home - Axum SSE Demo</title>
	<meta name="description" content="Real-time time updates via Server-Sent Events" />
</svelte:head>

<div class="container">
	<div class="hero">
		<h1>Real-Time Clock</h1>
		<p class="subtitle">
			Live time updates every 10 seconds via Server-Sent Events (SSE)
		</p>
	</div>
	
	<div class="time-display">
		<div class="time-card">
			<div class="time-label">Current Time (UK Format)</div>
			<div class="time-value" class:connected={isConnected && !isStale} class:disconnected={!isConnected || isStale}>
				{currentTime}
			</div>
			{#if lastUpdated}
				<div class="last-updated" class:stale={isStale}>
					Last updated: {relativeTime}
					{#if isStale}
						<span class="stale-warning">⚠️ Data may be stale</span>
					{/if}
				</div>
			{/if}
			{#if !isConnected && $dateTimeStore.currentTime}
				<div class="offline-notice">
					📱 Showing cached time - will update when reconnected
				</div>
			{/if}
		</div>
	</div>
	
	<div class="connection-panel">
		<div class="status-section">
			<h3>Connection Status</h3>
			<div class="status-info">
				<div class="status-row">
					<span class="label">Status:</span>
					<span class="value status-{$connectionState.connecting ? 'connecting' : ($connectionState.connected ? 'connected' : 'disconnected')}">{statusDisplay}</span>
				</div>
				<div class="status-row">
					<span class="label">Connection ID:</span>
					<span class="value">{$connectionState.connectionId || 'None'}</span>
				</div>
				<div class="status-row">
					<span class="label">Failed Attempts:</span>
					<span class="value">{$connectionState.failedAttempts}</span>
				</div>
				{#if !isConnected && reconnectDelay > 0}
					<div class="status-row">
						<span class="label">Reconnect Delay:</span>
						<span class="value">{reconnectDelay}ms</span>
					</div>
				{/if}
			</div>
		</div>
		
		<div class="controls">
			<h3>Controls</h3>
			<div class="button-group">
				{#if isConnected}
					<button class="btn btn-danger" on:click={disconnect}>
						Disconnect
					</button>
				{:else}
					<button class="btn btn-primary" on:click={connectToTimeStream}>
						Connect
					</button>
				{/if}
				
				<button 
					class="btn btn-secondary" 
					on:click={manualBroadcast}
					disabled={!isConnected}
				>
					Test Broadcast
				</button>
				
				<button 
					class="btn btn-secondary" 
					on:click={clearStoredTime}
					disabled={!$dateTimeStore.currentTime}
				>
					Clear Cached Time
				</button>
			</div>
		</div>
		
		<div class="status-section">
			<h3>Data Persistence</h3>
			<div class="status-info">
				<div class="status-row">
					<span class="label">Cached Time:</span>
					<span class="value">{$dateTimeStore.currentTime || 'None'}</span>
				</div>
				<div class="status-row">
					<span class="label">Cache Status:</span>
					<span class="value status-{isStale ? 'disconnected' : 'connected'}">
						{isStale ? 'Stale' : 'Fresh'}
					</span>
				</div>
				<div class="status-row">
					<span class="label">Last Update:</span>
					<span class="value">{relativeTime}</span>
				</div>
				<div class="status-row">
					<span class="label">SPA Mode:</span>
					<span class="value status-connected">✓ Client-side routing active</span>
				</div>
			</div>
		</div>
	</div>
	
	<div class="info-section">
		<h3>About This Demo</h3>
		<div class="info-grid">
			<div class="info-card">
				<h4>🔄 Real-Time Updates</h4>
				<p>
					Time updates are pushed from the server every 10 seconds using Server-Sent Events (SSE),
					providing a persistent connection for live data streaming.
				</p>
			</div>
			
			<div class="info-card">
				<h4>🇬🇧 UK Date Format</h4>
				<p>
					All timestamps are formatted in UK standard (DD/MM/YYYY HH:MM:SS) using Rust's
					chrono library for accurate timezone handling.
				</p>
			</div>
			
			<div class="info-card">
				<h4>🔄 Auto-Reconnection</h4>
				<p>
					The connection automatically reconnects on failures with exponential backoff,
					ensuring reliable data delivery even during network interruptions.
				</p>
			</div>
			
			<div class="info-card">
				<h4>⚡ Performance</h4>
				<p>
					Built with Rust and Axum for high-performance backend processing,
					supporting 1000+ concurrent SSE connections.
				</p>
			</div>
		</div>
	</div>
</div>

<style>
	.container {
		max-width: 1200px;
		margin: 0 auto;
		padding: 0 2rem;
	}
	
	.hero {
		text-align: center;
		margin-bottom: 4rem;
	}
	
	.hero h1 {
		font-size: 3rem;
		margin: 0 0 1rem 0;
		background: var(--accent-gradient);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		font-weight: 800;
		letter-spacing: -0.02em;
		line-height: 1.1;
	}
	
	.subtitle {
		font-size: 1.25rem;
		color: var(--text-secondary);
		margin: 0;
		font-weight: 400;
	}
	
	.time-display {
		display: flex;
		justify-content: center;
		margin-bottom: 4rem;
	}
	
	.time-card {
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 24px;
		padding: 3rem 2rem;
		box-shadow: var(--shadow-lg);
		text-align: center;
		min-width: 450px;
		position: relative;
		overflow: hidden;
	}
	
	.time-card::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 2px;
		background: var(--accent-gradient);
	}
	
	.time-label {
		font-size: 1rem;
		color: var(--text-muted);
		margin-bottom: 1.5rem;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
	
	.time-value {
		font-size: 3rem;
		font-weight: 700;
		font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
		margin-bottom: 1rem;
		padding: 1.5rem;
		border-radius: 16px;
		transition: all 0.3s ease;
		letter-spacing: -0.02em;
	}
	
	.time-value.connected {
		background: rgba(16, 185, 129, 0.1);
		color: var(--success);
		border: 2px solid rgba(16, 185, 129, 0.3);
		box-shadow: 0 0 20px rgba(16, 185, 129, 0.1);
	}
	
	.time-value.disconnected {
		background: rgba(239, 68, 68, 0.1);
		color: var(--error);
		border: 2px solid rgba(239, 68, 68, 0.3);
		box-shadow: 0 0 20px rgba(239, 68, 68, 0.1);
	}
	
	.last-updated {
		font-size: 0.875rem;
		color: var(--text-muted);
		margin-top: 1rem;
		font-weight: 500;
	}
	
	.last-updated.stale {
		color: var(--warning);
	}
	
	.stale-warning {
		display: block;
		font-size: 0.75rem;
		color: var(--warning);
		margin-top: 0.5rem;
		font-weight: 600;
	}
	
	.offline-notice {
		font-size: 0.875rem;
		color: var(--accent-primary);
		margin-top: 1rem;
		padding: 0.75rem;
		background: rgba(109, 40, 217, 0.1);
		border: 1px solid rgba(109, 40, 217, 0.3);
		border-radius: 8px;
		font-weight: 500;
	}
	
	.connection-panel {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr;
		gap: 2rem;
		margin-bottom: 4rem;
	}
	
	.status-section,
	.controls {
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 20px;
		padding: 2.5rem;
		box-shadow: var(--shadow-lg);
		position: relative;
		overflow: hidden;
	}
	
	.status-section::before,
	.controls::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 2px;
		background: var(--accent-gradient);
	}
	
	.status-section h3,
	.controls h3 {
		margin: 0 0 2rem 0;
		color: var(--text-primary);
		font-weight: 700;
		font-size: 1.375rem;
		letter-spacing: -0.02em;
	}
	
	.status-info {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}
	
	.status-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 0;
		border-bottom: 1px solid var(--border);
	}
	
	.status-row:last-child {
		border-bottom: none;
	}
	
	.label {
		font-weight: 600;
		color: var(--text-secondary);
		font-size: 0.925rem;
	}
	
	.value {
		font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
		font-weight: 600;
		font-size: 0.925rem;
	}
	
	.status-connected {
		color: var(--success);
	}
	
	.status-connecting {
		color: var(--warning);
	}
	
	.status-disconnected {
		color: var(--error);
	}
	
	.button-group {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
	}
	
	.btn {
		padding: 0.875rem 1.75rem;
		border: none;
		border-radius: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.3s ease;
		font-size: 0.925rem;
		position: relative;
		overflow: hidden;
	}
	
	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		transform: none;
	}
	
	.btn-primary {
		background: var(--accent-gradient);
		color: white;
		border: 1px solid transparent;
	}
	
	.btn-primary:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: var(--shadow-xl);
		filter: brightness(1.1);
	}
	
	.btn-danger {
		background: linear-gradient(135deg, var(--error), #dc2626);
		color: white;
		border: 1px solid transparent;
	}
	
	.btn-danger:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(239, 68, 68, 0.3);
		filter: brightness(1.1);
	}
	
	.btn-secondary {
		background: var(--bg-tertiary);
		color: var(--text-primary);
		border: 1px solid var(--border);
	}
	
	.btn-secondary:hover:not(:disabled) {
		transform: translateY(-2px);
		background: var(--bg-secondary);
		border-color: var(--accent-primary);
		box-shadow: var(--shadow-lg);
	}
	
	.info-section {
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 24px;
		padding: 3rem;
		box-shadow: var(--shadow-lg);
		position: relative;
		overflow: hidden;
	}
	
	.info-section::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 3px;
		background: var(--accent-gradient);
	}
	
	.info-section h3 {
		margin: 0 0 3rem 0;
		color: var(--text-primary);
		font-weight: 800;
		text-align: center;
		font-size: 2rem;
		letter-spacing: -0.02em;
	}
	
	.info-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		gap: 2rem;
	}
	
	.info-card {
		padding: 2rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border);
		border-radius: 16px;
		transition: all 0.3s ease;
		position: relative;
		overflow: hidden;
	}
	
	.info-card::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		width: 4px;
		height: 100%;
		background: var(--accent-gradient);
	}
	
	.info-card:hover {
		transform: translateY(-4px);
		box-shadow: var(--shadow-xl);
		border-color: var(--accent-primary);
	}
	
	.info-card h4 {
		margin: 0 0 1.25rem 0;
		color: var(--text-primary);
		font-size: 1.125rem;
		font-weight: 700;
		letter-spacing: -0.01em;
	}
	
	.info-card p {
		margin: 0;
		color: var(--text-secondary);
		line-height: 1.7;
		font-size: 0.95rem;
	}
	
	/* Responsive design */
	@media (max-width: 968px) {
		.connection-panel {
			grid-template-columns: 1fr;
		}
		
		.info-grid {
			grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		}
	}
	
	@media (max-width: 768px) {
		.container {
			padding: 0 1rem;
		}
		
		.hero {
			margin-bottom: 3rem;
		}
		
		.hero h1 {
			font-size: 2.25rem;
		}
		
		.subtitle {
			font-size: 1.125rem;
		}
		
		.time-display {
			margin-bottom: 3rem;
		}
		
		.time-card {
			min-width: unset;
			width: 100%;
			padding: 2rem 1.5rem;
		}
		
		.time-value {
			font-size: 2.25rem;
		}
		
		.connection-panel {
			gap: 1.5rem;
			margin-bottom: 3rem;
		}
		
		.status-section,
		.controls {
			padding: 2rem;
		}
		
		.button-group {
			flex-direction: column;
		}
		
		.btn {
			width: 100%;
			justify-content: center;
		}
		
		.info-section {
			padding: 2rem;
		}
		
		.info-section h3 {
			font-size: 1.75rem;
			margin-bottom: 2rem;
		}
		
		.info-grid {
			grid-template-columns: 1fr;
			gap: 1.5rem;
		}
	}
	
	@media (max-width: 480px) {
		.hero h1 {
			font-size: 1.875rem;
		}
		
		.time-card {
			padding: 1.5rem 1rem;
		}
		
		.time-value {
			font-size: 1.875rem;
			padding: 1rem;
		}
		
		.status-section,
		.controls,
		.info-section {
			padding: 1.5rem;
		}
		
		.info-card {
			padding: 1.5rem;
		}
	}
</style>