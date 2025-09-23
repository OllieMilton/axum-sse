<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import Chart from 'chart.js/auto';
	import type { Chart as ChartType } from 'chart.js';
	
	// Types for the status data based on the Rust backend
	interface ServerMetrics {
		timestamp: string;
		memory_usage: {
			total_bytes: number;
			used_bytes: number;
			available_bytes: number;
			usage_percentage: number;
		};
		cpu_usage: {
			usage_percentage: number;
			core_count: number;
			load_average: {
				one_minute: number;
				five_minute: number;
				fifteen_minute: number;
			};
		};
		uptime: {
			secs: number;
			nanos: number;
		};
		network_metrics: {
			bytes_sent: number;
			bytes_received: number;
			packets_sent: number;
			packets_received: number;
			active_connections: number;
		};
	}
	
	interface OsInfo {
		name: string;
		version: string;
		architecture: string;
		kernel_version: string;
		long_description: string;
	}
	
	interface ServerInfo {
		hostname: string;
		version: string;
		start_time: string;
		environment: string;
		os_info: OsInfo;
	}
	
	interface StatusData {
		server_metrics: ServerMetrics;
		collection_interval_seconds: number;
		server_info: ServerInfo;
	}
	
	interface MetricsEvent {
		event_type: string;
		data: StatusData;
		sequence: number;
		timestamp: string;
	}
	
	// Component state
	let statusData: StatusData | null = null;
	let eventSource: EventSource | null = null;
	let isConnected = false;
	let connectionError: string | null = null;
	let lastUpdate: Date | null = null;
	
	// Chart instances
	let memoryChart: ChartType | null = null;
	let cpuChart: ChartType | null = null;
	let networkChart: ChartType | null = null;
	
	// Chart canvas elements
	let memoryChartCanvas: HTMLCanvasElement;
	let cpuChartCanvas: HTMLCanvasElement;
	let networkChartCanvas: HTMLCanvasElement;
	
	// Historical data for charts (keep last 20 data points)
	let memoryHistory: { time: string; value: number }[] = [];
	let cpuHistory: { time: string; value: number }[] = [];
	let networkHistory: { time: string; sent: number; received: number }[] = [];
	
	const MAX_HISTORY_POINTS = 20;
	
	// Utility functions
	function formatBytes(bytes: number): string {
		const sizes = ['Bytes', 'KB', 'MB', 'GB'];
		if (bytes === 0) return '0 Bytes';
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
	}
	
	function formatUptime(uptime: { secs: number; nanos: number }): string {
		const totalSeconds = uptime.secs;
		const days = Math.floor(totalSeconds / 86400);
		const hours = Math.floor((totalSeconds % 86400) / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		
		if (days > 0) {
			return `${days}d ${hours}h ${minutes}m`;
		} else if (hours > 0) {
			return `${hours}h ${minutes}m`;
		} else {
			return `${minutes}m`;
		}
	}
	
	function getTimeString(timestamp: string): string {
		return new Date(timestamp).toLocaleTimeString();
	}
	
	// Chart initialization
	function initializeCharts() {
		if (!browser) return;
		
		// Memory usage chart
		if (memoryChartCanvas && !memoryChart) {
			memoryChart = new Chart(memoryChartCanvas, {
				type: 'line',
				data: {
					labels: [],
					datasets: [{
						label: 'Memory Usage (%)',
						data: [],
						borderColor: 'rgb(255, 99, 132)',
						backgroundColor: 'rgba(255, 99, 132, 0.2)',
						tension: 0.1
					}]
				},
				options: {
					responsive: true,
					maintainAspectRatio: false,
					scales: {
						y: {
							beginAtZero: true,
							max: 100
						}
					},
					plugins: {
						legend: {
							display: false
						}
					}
				}
			});
		}
		
		// CPU usage chart
		if (cpuChartCanvas && !cpuChart) {
			cpuChart = new Chart(cpuChartCanvas, {
				type: 'line',
				data: {
					labels: [],
					datasets: [{
						label: 'CPU Usage (%)',
						data: [],
						borderColor: 'rgb(54, 162, 235)',
						backgroundColor: 'rgba(54, 162, 235, 0.2)',
						tension: 0.1
					}]
				},
				options: {
					responsive: true,
					maintainAspectRatio: false,
					scales: {
						y: {
							beginAtZero: true,
							max: 100
						}
					},
					plugins: {
						legend: {
							display: false
						}
					}
				}
			});
		}
		
		// Network traffic chart
		if (networkChartCanvas && !networkChart) {
			networkChart = new Chart(networkChartCanvas, {
				type: 'line',
				data: {
					labels: [],
					datasets: [
						{
							label: 'Sent (MB/s)',
							data: [],
							borderColor: 'rgb(75, 192, 192)',
							backgroundColor: 'rgba(75, 192, 192, 0.2)',
							tension: 0.1
						},
						{
							label: 'Received (MB/s)',
							data: [],
							borderColor: 'rgb(153, 102, 255)',
							backgroundColor: 'rgba(153, 102, 255, 0.2)',
							tension: 0.1
						}
					]
				},
				options: {
					responsive: true,
					maintainAspectRatio: false,
					scales: {
						y: {
							beginAtZero: true
						}
					}
				}
			});
		}
	}
	
	// Update charts with new data
	function updateCharts(metrics: ServerMetrics) {
		const timestamp = getTimeString(metrics.timestamp);
		
		// Update memory chart
		if (memoryChart) {
			memoryHistory.push({
				time: timestamp,
				value: metrics.memory_usage.usage_percentage
			});
			
			if (memoryHistory.length > MAX_HISTORY_POINTS) {
				memoryHistory.shift();
			}
			
			memoryChart.data.labels = memoryHistory.map(d => d.time);
			memoryChart.data.datasets[0].data = memoryHistory.map(d => d.value);
			memoryChart.update('none');
		}
		
		// Update CPU chart
		if (cpuChart) {
			cpuHistory.push({
				time: timestamp,
				value: metrics.cpu_usage.usage_percentage
			});
			
			if (cpuHistory.length > MAX_HISTORY_POINTS) {
				cpuHistory.shift();
			}
			
			cpuChart.data.labels = cpuHistory.map(d => d.time);
			cpuChart.data.datasets[0].data = cpuHistory.map(d => d.value);
			cpuChart.update('none');
		}
		
		// Update network chart (convert bytes to MB/s approximation)
		if (networkChart) {
			const sentMBps = metrics.network_metrics.bytes_sent / (1024 * 1024);
			const receivedMBps = metrics.network_metrics.bytes_received / (1024 * 1024);
			
			networkHistory.push({
				time: timestamp,
				sent: sentMBps,
				received: receivedMBps
			});
			
			if (networkHistory.length > MAX_HISTORY_POINTS) {
				networkHistory.shift();
			}
			
			networkChart.data.labels = networkHistory.map(d => d.time);
			networkChart.data.datasets[0].data = networkHistory.map(d => d.sent);
			networkChart.data.datasets[1].data = networkHistory.map(d => d.received);
			networkChart.update('none');
		}
	}
	
	// SSE connection management
	function connectToStatusStream() {
		if (eventSource) {
			eventSource.close();
		}
		
		isConnected = false;
		connectionError = null;
		
		console.log('Connecting to server status stream...');
		
		try {
			eventSource = new EventSource('/api/server-status-stream?interval=5');
			
			eventSource.onopen = () => {
				console.log('Server status stream connected');
				isConnected = true;
				connectionError = null;
			};
			
			eventSource.addEventListener('status-update', (event) => {
				try {
					const metricsEvent: MetricsEvent = JSON.parse(event.data);
					statusData = metricsEvent.data;
					lastUpdate = new Date();
					
					// Update charts with new data
					updateCharts(statusData.server_metrics);
					
					// Clear any previous connection errors
					connectionError = null;
				} catch (error) {
					console.error('Error parsing status update:', error);
				}
			});
			
			// Handle error events from the server
			eventSource.addEventListener('error', (event) => {
				console.warn('Server sent error event, waiting for valid metrics...');
				// Don't treat server error events as connection failures
				// The server will send proper status-update events once metrics are available
			});
			
			eventSource.onerror = (error) => {
				console.error('SSE connection error:', error);
				
				// Only treat as connection failure if EventSource is in error state
				if (eventSource && eventSource.readyState === EventSource.CLOSED) {
					isConnected = false;
					connectionError = 'Connection lost. Attempting to reconnect...';
					
					// Attempt to reconnect after 3 seconds
					setTimeout(() => {
						if (!isConnected) {
							connectToStatusStream();
						}
					}, 3000);
				}
			};
		} catch (error) {
			console.error('Failed to connect to status stream:', error);
			connectionError = 'Failed to connect to server';
		}
	}
	
	// Load initial data
	async function loadInitialData() {
		try {
			const response = await fetch('/api/server-status');
			if (response.ok) {
				const data = await response.json();
				statusData = data.data;
				lastUpdate = new Date();
			}
		} catch (error) {
			console.error('Failed to load initial status data:', error);
		}
	}
	
	// Component lifecycle
	onMount(async () => {
		await loadInitialData();
		initializeCharts();
		connectToStatusStream();
	});
	
	onDestroy(() => {
		if (eventSource) {
			eventSource.close();
		}
		
		// Destroy chart instances
		if (memoryChart) {
			memoryChart.destroy();
		}
		if (cpuChart) {
			cpuChart.destroy();
		}
		if (networkChart) {
			networkChart.destroy();
		}
	});
	
	// Reactive statements
	$: healthStatus = statusData ? getHealthStatus(statusData.server_metrics) : 'unknown';
	
	function getHealthStatus(metrics: ServerMetrics): string {
		const memoryUsage = metrics.memory_usage.usage_percentage;
		const cpuUsage = metrics.cpu_usage.usage_percentage;
		
		if (memoryUsage > 90 || cpuUsage > 90) {
			return 'critical';
		} else if (memoryUsage > 75 || cpuUsage > 75) {
			return 'warning';
		} else {
			return 'healthy';
		}
	}
</script>

<svelte:head>
	<title>Server Status - Axum SSE Demo</title>
	<meta name="description" content="Real-time server metrics and status monitoring" />
</svelte:head>

<div class="status-page">
	<div class="page-header">
		<h1>Server Status</h1>
		<div class="status-indicator" class:healthy={healthStatus === 'healthy'} 
			 class:warning={healthStatus === 'warning'} 
			 class:critical={healthStatus === 'critical'}>
			<div class="status-dot"></div>
			<span class="status-text">{healthStatus.toUpperCase()}</span>
		</div>
	</div>
	
	<!-- Connection Status -->
	<div class="connection-status" class:connected={isConnected} class:disconnected={!isConnected}>
		{#if isConnected}
			<span class="connection-icon">🟢</span>
			<span>Connected to real-time stream</span>
			{#if lastUpdate}
				<span class="last-update">Last update: {lastUpdate.toLocaleTimeString()}</span>
			{/if}
		{:else}
			<span class="connection-icon">🔴</span>
			<span>{connectionError || 'Disconnected'}</span>
		{/if}
	</div>

	{#if statusData}
		<!-- Server Information -->
		<div class="server-info-section">
			<h2>Server Information</h2>
			<div class="info-grid">
				<div class="info-item">
					<span class="label">Hostname:</span>
					<span class="value">{statusData.server_info.hostname}</span>
				</div>
				<div class="info-item">
					<span class="label">Version:</span>
					<span class="value">{statusData.server_info.version}</span>
				</div>
				<div class="info-item">
					<span class="label">Environment:</span>
					<span class="value">{statusData.server_info.environment}</span>
				</div>
				<div class="info-item">
					<span class="label">Uptime:</span>
					<span class="value">{formatUptime(statusData.server_metrics.uptime)}</span>
				</div>
				<div class="info-item">
					<span class="label">Operating System:</span>
					<span class="value">{statusData.server_info.os_info.name}</span>
				</div>
				<div class="info-item">
					<span class="label">OS Version:</span>
					<span class="value">{statusData.server_info.os_info.version}</span>
				</div>
				<div class="info-item">
					<span class="label">Architecture:</span>
					<span class="value">{statusData.server_info.os_info.architecture}</span>
				</div>
				<div class="info-item">
					<span class="label">Kernel:</span>
					<span class="value">{statusData.server_info.os_info.kernel_version}</span>
				</div>
			</div>
		</div>

		<!-- Metrics Dashboard -->
		<div class="metrics-dashboard">
			<h2>Real-time Metrics</h2>
			
			<!-- Memory Usage -->
			<div class="metric-card">
				<h3>Memory Usage</h3>
				<div class="metric-content">
					<div class="metric-summary">
						<div class="percentage" class:high={statusData.server_metrics.memory_usage.usage_percentage > 75}>
							{statusData.server_metrics.memory_usage.usage_percentage.toFixed(1)}%
						</div>
						<div class="details">
							<div>Used: {formatBytes(statusData.server_metrics.memory_usage.used_bytes)}</div>
							<div>Available: {formatBytes(statusData.server_metrics.memory_usage.available_bytes)}</div>
							<div>Total: {formatBytes(statusData.server_metrics.memory_usage.total_bytes)}</div>
						</div>
					</div>
					<div class="chart-container">
						<canvas bind:this={memoryChartCanvas}></canvas>
					</div>
				</div>
			</div>

			<!-- CPU Usage -->
			<div class="metric-card">
				<h3>CPU Usage</h3>
				<div class="metric-content">
					<div class="metric-summary">
						<div class="percentage" class:high={statusData.server_metrics.cpu_usage.usage_percentage > 75}>
							{statusData.server_metrics.cpu_usage.usage_percentage.toFixed(1)}%
						</div>
						<div class="details">
							<div>Cores: {statusData.server_metrics.cpu_usage.core_count}</div>
							<div>Load 1m: {statusData.server_metrics.cpu_usage.load_average.one_minute.toFixed(2)}</div>
							<div>Load 5m: {statusData.server_metrics.cpu_usage.load_average.five_minute.toFixed(2)}</div>
							<div>Load 15m: {statusData.server_metrics.cpu_usage.load_average.fifteen_minute.toFixed(2)}</div>
						</div>
					</div>
					<div class="chart-container">
						<canvas bind:this={cpuChartCanvas}></canvas>
					</div>
				</div>
			</div>

			<!-- Network Metrics -->
			<div class="metric-card">
				<h3>Network Activity</h3>
				<div class="metric-content">
					<div class="metric-summary">
						<div class="details">
							<div>Sent: {formatBytes(statusData.server_metrics.network_metrics.bytes_sent)}</div>
							<div>Received: {formatBytes(statusData.server_metrics.network_metrics.bytes_received)}</div>
							<div>Packets Sent: {statusData.server_metrics.network_metrics.packets_sent.toLocaleString()}</div>
							<div>Packets Received: {statusData.server_metrics.network_metrics.packets_received.toLocaleString()}</div>
							<div>Active Connections: {statusData.server_metrics.network_metrics.active_connections}</div>
						</div>
					</div>
					<div class="chart-container">
						<canvas bind:this={networkChartCanvas}></canvas>
					</div>
				</div>
			</div>
		</div>
	{:else}
		<div class="loading">
			<div class="loading-spinner"></div>
			<p>Loading server status...</p>
		</div>
	{/if}
</div>

<style>
	.status-page {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem;
	}

	.page-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid var(--border);
	}

	.page-header h1 {
		margin: 0;
		color: var(--text-primary);
	}

	.status-indicator {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		border-radius: 0.5rem;
		font-weight: 600;
		text-transform: uppercase;
		font-size: 0.875rem;
	}

	.status-indicator.healthy {
		background-color: rgba(34, 197, 94, 0.1);
		color: rgb(34, 197, 94);
	}

	.status-indicator.warning {
		background-color: rgba(251, 191, 36, 0.1);
		color: rgb(251, 191, 36);
	}

	.status-indicator.critical {
		background-color: rgba(239, 68, 68, 0.1);
		color: rgb(239, 68, 68);
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: currentColor;
	}

	.connection-status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 1rem;
		margin-bottom: 2rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
	}

	.connection-status.connected {
		background-color: rgba(34, 197, 94, 0.1);
		color: rgb(34, 197, 94);
	}

	.connection-status.disconnected {
		background-color: rgba(239, 68, 68, 0.1);
		color: rgb(239, 68, 68);
	}

	.last-update {
		margin-left: auto;
		font-size: 0.75rem;
		opacity: 0.8;
	}

	.server-info-section {
		margin-bottom: 2rem;
	}

	.server-info-section h2 {
		margin-bottom: 1rem;
		color: var(--text-primary);
	}

	.info-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
	}

	.info-item {
		display: flex;
		justify-content: space-between;
		padding: 0.75rem;
		background-color: var(--surface);
		border-radius: 0.5rem;
		border: 1px solid var(--border);
	}

	.info-item .label {
		font-weight: 600;
		color: var(--text-secondary);
	}

	.info-item .value {
		color: var(--text-primary);
	}

	.metrics-dashboard h2 {
		margin-bottom: 1.5rem;
		color: var(--text-primary);
	}

	.metric-card {
		background-color: var(--surface);
		border: 1px solid var(--border);
		border-radius: 0.75rem;
		padding: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.metric-card h3 {
		margin: 0 0 1rem 0;
		color: var(--text-primary);
	}

	.metric-content {
		display: grid;
		grid-template-columns: 200px 1fr;
		gap: 1.5rem;
		align-items: center;
	}

	.metric-summary {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.percentage {
		font-size: 2rem;
		font-weight: 700;
		color: var(--accent);
	}

	.percentage.high {
		color: rgb(239, 68, 68);
	}

	.details {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-size: 0.875rem;
		color: var(--text-secondary);
	}

	.chart-container {
		height: 200px;
		position: relative;
	}

	.loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 4rem;
		gap: 1rem;
	}

	.loading-spinner {
		width: 40px;
		height: 40px;
		border: 4px solid var(--border);
		border-top: 4px solid var(--accent);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		0% { transform: rotate(0deg); }
		100% { transform: rotate(360deg); }
	}

	@media (max-width: 768px) {
		.status-page {
			padding: 1rem;
		}

		.page-header {
			flex-direction: column;
			align-items: flex-start;
			gap: 1rem;
		}

		.metric-content {
			grid-template-columns: 1fr;
		}

		.info-grid {
			grid-template-columns: 1fr;
		}
	}
</style>