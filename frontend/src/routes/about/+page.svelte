<script lang="ts">
	import { onMount } from 'svelte';
	
	let serverInfo: any = null;
	let loading = true;
	let error = '';
	
	async function fetchServerInfo() {
		try {
			const response = await fetch('/api/status');
			if (response.ok) {
				serverInfo = await response.json();
			} else {
				error = `Failed to fetch server info: ${response.statusText}`;
			}
		} catch (err) {
			error = `Network error: ${err instanceof Error ? err.message : 'Unknown error'}`;
		} finally {
			loading = false;
		}
	}
	
	onMount(() => {
		fetchServerInfo();
	});
</script>

<svelte:head>
	<title>About - Axum SSE Demo</title>
	<meta name="description" content="Learn about the Axum SSE demo implementation" />
</svelte:head>

<div class="container">
	<div class="hero">
		<h1>About This Demo</h1>
		<p class="subtitle">
			A real-time web application showcasing Server-Sent Events with Rust and SvelteKit
		</p>
	</div>
	
	<div class="content-grid">
		<!-- Project Overview -->
		<section class="card">
			<h2>🚀 Project Overview</h2>
			<p>
				This demonstration showcases a modern real-time web application built with cutting-edge
				technologies. The backend is powered by <strong>Rust</strong> and <strong>Axum</strong>,
				providing high-performance HTTP handling and Server-Sent Events (SSE) streaming.
			</p>
			<p>
				The frontend is built with <strong>SvelteKit</strong> and <strong>TypeScript</strong>,
				offering a reactive user interface with type safety and excellent developer experience.
			</p>
		</section>

		<!-- Technology Stack -->
		<section class="card">
			<h2>🛠️ Technology Stack</h2>
			<div class="tech-grid">
				<div class="tech-item">
					<h3>Backend</h3>
					<ul>
						<li><strong>Rust 1.75+</strong> - Systems programming language</li>
						<li><strong>Axum 0.7</strong> - Modern async web framework</li>
						<li><strong>Tokio</strong> - Async runtime</li>
						<li><strong>Chrono</strong> - Date/time handling</li>
						<li><strong>Serde</strong> - Serialization framework</li>
					</ul>
				</div>
				<div class="tech-item">
					<h3>Frontend</h3>
					<ul>
						<li><strong>SvelteKit</strong> - Full-stack web framework</li>
						<li><strong>TypeScript</strong> - Type-safe JavaScript</li>
						<li><strong>Vite</strong> - Fast build tool</li>
						<li><strong>CSS3</strong> - Modern styling</li>
						<li><strong>EventSource API</strong> - SSE client</li>
					</ul>
				</div>
			</div>
		</section>

		<!-- Features -->
		<section class="card">
			<h2>✨ Key Features</h2>
			<div class="feature-grid">
				<div class="feature-item">
					<h3>🔄 Real-Time Updates</h3>
					<p>Server-Sent Events provide persistent connections for live data streaming every 10 seconds.</p>
				</div>
				<div class="feature-item">
					<h3>🇬🇧 UK Date Format</h3>
					<p>All timestamps use UK standard format (DD/MM/YYYY HH:MM:SS) with proper timezone handling.</p>
				</div>
				<div class="feature-item">
					<h3>🔄 Auto-Reconnection</h3>
					<p>Intelligent reconnection with exponential backoff ensures reliable connectivity.</p>
				</div>
				<div class="feature-item">
					<h3>⚡ High Performance</h3>
					<p>Rust backend supports 1000+ concurrent connections with minimal resource usage.</p>
				</div>
				<div class="feature-item">
					<h3>🛡️ Type Safety</h3>
					<p>End-to-end type safety with Rust and TypeScript ensures robust code.</p>
				</div>
				<div class="feature-item">
					<h3>📱 Responsive Design</h3>
					<p>Mobile-first design works seamlessly across all device sizes.</p>
				</div>
			</div>
		</section>

		<!-- Server Status -->
		<section class="card">
			<h2>📊 Server Status</h2>
			{#if loading}
				<div class="loading-state">
					<div class="spinner"></div>
					<p>Loading server information...</p>
				</div>
			{:else if error}
				<div class="error-state">
					<p class="error">❌ {error}</p>
					<button class="btn btn-primary" on:click={fetchServerInfo}>
						Retry
					</button>
				</div>
			{:else if serverInfo}
				<div class="server-info">
					<div class="info-row">
						<span class="label">Server Status:</span>
						<span class="value status-{serverInfo.status?.toLowerCase() || 'unknown'}">
							{serverInfo.status || 'Unknown'}
						</span>
					</div>
					<div class="info-row">
						<span class="label">Uptime:</span>
						<span class="value">{serverInfo.uptime || 'Unknown'}</span>
					</div>
					<div class="info-row">
						<span class="label">Active Connections:</span>
						<span class="value">{serverInfo.active_connections || 0}</span>
					</div>
					<div class="info-row">
						<span class="label">Total Broadcasts:</span>
						<span class="value">{serverInfo.total_broadcasts || 0}</span>
					</div>
					<div class="info-row">
						<span class="label">Version:</span>
						<span class="value">{serverInfo.version || '0.1.0'}</span>
					</div>
					<div class="info-row">
						<span class="label">Build Mode:</span>
						<span class="value">{serverInfo.build_mode || 'Development'}</span>
					</div>
				</div>
			{/if}
		</section>

		<!-- API Documentation -->
		<section class="card">
			<h2>📡 API Endpoints</h2>
			<div class="api-docs">
				<div class="endpoint">
					<div class="method get">GET</div>
					<div class="path">/api/time-stream</div>
					<div class="description">Server-Sent Events stream with time updates every 10 seconds</div>
				</div>
				<div class="endpoint">
					<div class="method get">GET</div>
					<div class="path">/api/health</div>
					<div class="description">Basic health check endpoint</div>
				</div>
				<div class="endpoint">
					<div class="method get">GET</div>
					<div class="path">/api/status</div>
					<div class="description">Detailed server status and metrics</div>
				</div>
				<div class="endpoint">
					<div class="method post">POST</div>
					<div class="path">/api/broadcast</div>
					<div class="description">Manual time broadcast trigger for testing</div>
				</div>
			</div>
		</section>

		<!-- Implementation Details -->
		<section class="card">
			<h2>🏗️ Implementation Details</h2>
			<div class="details-content">
				<h3>Architecture</h3>
				<p>
					The application follows a layered architecture with clear separation of concerns:
				</p>
				<ul>
					<li><strong>Models</strong> - Data structures for time events and connection state</li>
					<li><strong>Services</strong> - Business logic for SSE broadcasting and static asset serving</li>
					<li><strong>Routes</strong> - HTTP handlers for API endpoints and page serving</li>
					<li><strong>Middleware</strong> - Cross-cutting concerns like CORS, logging, and security</li>
				</ul>

				<h3>Connection Management</h3>
				<p>
					The frontend maintains connection state through Svelte stores, providing reactive
					updates to the UI when connection status changes. Exponential backoff ensures
					graceful handling of network issues.
				</p>

				<h3>Testing</h3>
				<p>
					Comprehensive test suite with 41 passing tests covering unit tests for models
					and services, integration tests for routes and middleware, and end-to-end
					functionality verification.
				</p>
			</div>
		</section>
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
		margin-bottom: 3rem;
	}
	
	.hero h1 {
		font-size: 3rem;
		margin: 0 0 1rem 0;
		color: white;
		font-weight: 700;
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
	}
	
	.subtitle {
		font-size: 1.2rem;
		color: rgba(255, 255, 255, 0.9);
		margin: 0;
	}
	
	.content-grid {
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}
	
	.card {
		background: rgba(255, 255, 255, 0.95);
		border-radius: 1rem;
		padding: 2rem;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
	}
	
	.card h2 {
		margin: 0 0 1.5rem 0;
		color: #374151;
		font-weight: 600;
		font-size: 1.5rem;
	}
	
	.card h3 {
		margin: 1.5rem 0 1rem 0;
		color: #4c1d95;
		font-weight: 600;
		font-size: 1.2rem;
	}
	
	.card p {
		color: #6b7280;
		line-height: 1.6;
		margin: 0 0 1rem 0;
	}
	
	.tech-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 2rem;
	}
	
	.tech-item ul {
		margin: 1rem 0 0 0;
		padding-left: 1.5rem;
	}
	
	.tech-item li {
		color: #6b7280;
		line-height: 1.6;
		margin-bottom: 0.5rem;
	}
	
	.feature-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
	}
	
	.feature-item {
		padding: 1.5rem;
		background: #f9fafb;
		border-radius: 0.5rem;
		border-left: 4px solid #4c1d95;
	}
	
	.feature-item h3 {
		margin: 0 0 1rem 0;
		color: #374151;
		font-size: 1.1rem;
	}
	
	.feature-item p {
		margin: 0;
		font-size: 0.9rem;
	}
	
	.loading-state {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 2rem;
		justify-content: center;
	}
	
	.spinner {
		width: 24px;
		height: 24px;
		border: 3px solid #e5e7eb;
		border-top: 3px solid #4c1d95;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}
	
	@keyframes spin {
		0% { transform: rotate(0deg); }
		100% { transform: rotate(360deg); }
	}
	
	.error-state {
		text-align: center;
		padding: 2rem;
	}
	
	.error {
		color: #ef4444;
		font-weight: 600;
		margin-bottom: 1rem;
	}
	
	.server-info {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	
	.info-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem;
		background: #f9fafb;
		border-radius: 0.5rem;
	}
	
	.label {
		font-weight: 500;
		color: #6b7280;
	}
	
	.value {
		font-family: 'Courier New', monospace;
		font-weight: 600;
	}
	
	.status-healthy {
		color: #065f46;
	}
	
	.status-degraded {
		color: #d97706;
	}
	
	.status-unhealthy {
		color: #991b1b;
	}
	
	.api-docs {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	
	.endpoint {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: 1rem;
		align-items: center;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.5rem;
	}
	
	.method {
		padding: 0.25rem 0.75rem;
		border-radius: 0.25rem;
		font-weight: 600;
		font-size: 0.8rem;
		text-align: center;
		min-width: 60px;
	}
	
	.method.get {
		background: #ecfdf5;
		color: #065f46;
	}
	
	.method.post {
		background: #fef3c7;
		color: #92400e;
	}
	
	.path {
		font-family: 'Courier New', monospace;
		font-weight: 600;
		color: #374151;
	}
	
	.description {
		color: #6b7280;
		font-size: 0.9rem;
	}
	
	.details-content ul {
		margin: 1rem 0;
		padding-left: 1.5rem;
	}
	
	.details-content li {
		color: #6b7280;
		line-height: 1.6;
		margin-bottom: 0.5rem;
	}
	
	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 0.5rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
		font-size: 0.9rem;
	}
	
	.btn-primary {
		background: #4c1d95;
		color: white;
	}
	
	.btn-primary:hover {
		background: #5b21b6;
		transform: translateY(-1px);
	}
	
	/* Responsive design */
	@media (max-width: 768px) {
		.hero h1 {
			font-size: 2rem;
		}
		
		.tech-grid {
			grid-template-columns: 1fr;
		}
		
		.feature-grid {
			grid-template-columns: 1fr;
		}
		
		.endpoint {
			grid-template-columns: 1fr;
			gap: 0.5rem;
		}
		
		.method {
			justify-self: start;
		}
	}
</style>