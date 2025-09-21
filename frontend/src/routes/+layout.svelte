<script lang="ts">
	import { page } from '$app/stores';
	import { connectionState, connectionStatus } from '$lib/stores/connection';
	
	// Subscribe to connection state for banner display
	$: showBanner = $connectionStatus.shouldShowBanner;
	$: bannerClass = `connection-banner ${$connectionState.connecting ? 'connecting' : ($connectionState.connected ? 'connected' : 'disconnected')} ${showBanner ? '' : 'hidden'}`;
	$: bannerMessage = $connectionStatus.statusDisplay;
</script>

<svelte:head>
	<meta name="robots" content="noindex, nofollow" />
</svelte:head>

<!-- Connection Status Banner -->
<div class={bannerClass}>
	{bannerMessage}
</div>

<!-- Main Layout -->
<div class="app" class:has-banner={showBanner}>
	<!-- Navigation Header -->
	<header class="header">
		<nav class="nav">
			<div class="nav-brand">
				<h1>Axum SSE Demo</h1>
				<span class="subtitle">Real-time Server-Sent Events</span>
			</div>
			
			<div class="nav-links">
				<a 
					href="/" 
					class="nav-link" 
					class:active={$page.url.pathname === '/'}
				>
					Home
				</a>
				<a 
					href="/about" 
					class="nav-link" 
					class:active={$page.url.pathname === '/about'}
				>
					About
				</a>
			</div>
			
			<!-- Connection Status Indicator -->
			<div class="connection-status">
				<div class="status-dot" class:connected={$connectionState.connected}></div>
				<span class="status-text">{$connectionStatus.statusDisplay}</span>
			</div>
		</nav>
	</header>

	<!-- Main Content -->
	<main class="main">
		<slot />
	</main>

	<!-- Footer -->
	<footer class="footer">
		<p>&copy; 2025 Axum SSE Demo. Built with Rust, Axum, and SvelteKit.</p>
	</footer>
</div>

<style>
	.app {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		transition: margin-top 0.3s ease;
	}
	
	.app.has-banner {
		margin-top: 60px;
	}
	
	.header {
		background: rgba(26, 26, 46, 0.9);
		backdrop-filter: blur(20px);
		border-bottom: 1px solid var(--border);
		padding: 1.5rem 0;
		position: relative;
	}
	
	.header::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 1px;
		background: var(--accent-gradient);
	}
	
	.nav {
		max-width: 1200px;
		margin: 0 auto;
		padding: 0 2rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: 1.5rem;
	}
	
	.nav-brand h1 {
		margin: 0;
		font-size: 1.8rem;
		background: var(--accent-gradient);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		font-weight: 700;
		letter-spacing: -0.02em;
	}
	
	.subtitle {
		font-size: 0.85rem;
		color: var(--text-secondary);
		font-weight: 400;
		margin-top: 0.25rem;
		display: block;
	}
	
	.nav-links {
		display: flex;
		gap: 1rem;
		align-items: center;
	}
	
	.nav-link {
		text-decoration: none;
		color: var(--text-secondary);
		font-weight: 500;
		padding: 0.75rem 1.5rem;
		border-radius: 12px;
		transition: all 0.2s ease;
		position: relative;
		overflow: hidden;
	}
	
	.nav-link::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: var(--accent-gradient);
		opacity: 0;
		transition: opacity 0.2s ease;
		z-index: -1;
	}
	
	.nav-link:hover {
		color: var(--text-primary);
		transform: translateY(-1px);
	}
	
	.nav-link:hover::before {
		opacity: 0.1;
	}
	
	.nav-link.active {
		background: var(--accent-gradient);
		color: white;
		box-shadow: var(--shadow);
	}
	
	.nav-link.active::before {
		opacity: 1;
	}
	
	.connection-status {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 1.5rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: 16px;
		font-size: 0.9rem;
		backdrop-filter: blur(10px);
	}
	
	.status-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--error);
		transition: all 0.3s ease;
		box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.2);
	}
	
	.status-dot.connected {
		background: var(--success);
		box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.2);
	}
	
	.status-text {
		font-weight: 500;
		color: var(--text-primary);
	}
	
	.main {
		flex: 1;
		padding: 3rem 0;
	}
	
	.footer {
		background: var(--bg-secondary);
		border-top: 1px solid var(--border);
		padding: 2rem 0;
		text-align: center;
	}
	
	.footer p {
		margin: 0;
		color: var(--text-muted);
		font-size: 0.9rem;
	}
	
	/* Responsive design */
	@media (max-width: 768px) {
		.nav {
			flex-direction: column;
			text-align: center;
			gap: 1rem;
		}
		
		.nav-brand {
			order: 0;
		}
		
		.nav-links {
			order: 1;
			flex-wrap: wrap;
			justify-content: center;
		}
		
		.connection-status {
			order: 2;
		}
		
		.main {
			padding: 2rem 0;
		}
	}
	
	@media (max-width: 480px) {
		.nav {
			padding: 0 1rem;
		}
		
		.nav-brand h1 {
			font-size: 1.5rem;
		}
		
		.nav-links {
			gap: 0.5rem;
		}
		
		.nav-link {
			padding: 0.5rem 1rem;
			font-size: 0.9rem;
		}
	}
</style>