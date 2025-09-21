// Frontend Connection State management using Svelte stores
import { writable, derived, type Readable } from 'svelte/store';
import { dateTimeStore } from './datetime.js';

export interface ConnectionState {
	connected: boolean;
	connecting: boolean;
	lastPing: Date | null;
	connectionId: string | null;
	failedAttempts: number;
}

export interface ConnectionStatus {
	isConnected: boolean;
	isConnecting: boolean;
	isStale: boolean;
	shouldShowBanner: boolean;
	statusDisplay: string;
	reconnectionDelayMs: number;
}

// Create the connection state store
function createConnectionStore() {
	const initialState: ConnectionState = {
		connected: false,
		connecting: false,
		lastPing: null,
		connectionId: null,
		failedAttempts: 0
	};

	const { subscribe, set, update } = writable<ConnectionState>(initialState);

	return {
		subscribe,
		
		// Start connecting
		startConnecting: () => {
			update(state => ({
				...state,
				connecting: true,
				connected: false
			}));
		},
		
		// Connect with a new connection ID
		connect: (connectionId: string) => {
			update(state => ({
				...state,
				connected: true,
				connecting: false,
				lastPing: new Date(),
				connectionId,
				failedAttempts: 0
			}));
		},

		// Mark as disconnected and increment failed attempts
		disconnect: () => {
			update(state => ({
				...state,
				connected: false,
				connecting: false,
				lastPing: null,
				connectionId: null,
				failedAttempts: state.failedAttempts + 1
			}));
		},

		// Update ping timestamp
		ping: () => {
			update(state => 
				state.connected 
					? { ...state, lastPing: new Date() }
					: state
			);
		},

		// Update ping timestamp and datetime from SSE event
		pingWithTime: (timeString: string) => {
			// Update the datetime store with new time
			dateTimeStore.updateTime(timeString);
			
			// Update connection ping
			update(state => 
				state.connected 
					? { ...state, lastPing: new Date() }
					: state
			);
		},

		// Reset failed attempts (on successful reconnection)
		resetFailedAttempts: () => {
			update(state => ({ ...state, failedAttempts: 0 }));
		},

		// Reset to initial state
		reset: () => set(initialState)
	};
}

// Export the connection store instance
export const connectionState = createConnectionStore();

// Derived store for connection status with computed properties
export const connectionStatus: Readable<ConnectionStatus> = derived(
	connectionState,
	($connectionState) => {
		const now = new Date();
		const staleThreshold = 30 * 1000; // 30 seconds in milliseconds
		
		const isStale = $connectionState.lastPing 
			? (now.getTime() - $connectionState.lastPing.getTime()) > staleThreshold
			: true;

		const shouldShowBanner = !$connectionState.connected || isStale || $connectionState.connecting;

		let statusDisplay: string;
		if ($connectionState.connecting) {
			statusDisplay = 'Connecting...';
		} else if ($connectionState.connected && !isStale) {
			statusDisplay = 'Connected';
		} else if ($connectionState.connected && isStale) {
			statusDisplay = 'Connection unstable';
		} else {
			statusDisplay = 'Disconnected';
		}

		// Exponential backoff for reconnection delay
		const baseDelay = 1000; // 1 second
		const maxDelay = 30000; // 30 seconds
		const reconnectionDelayMs = Math.min(
			baseDelay * Math.pow(2, Math.min($connectionState.failedAttempts, 5)),
			maxDelay
		);

		return {
			isConnected: $connectionState.connected,
			isConnecting: $connectionState.connecting,
			isStale,
			shouldShowBanner,
			statusDisplay,
			reconnectionDelayMs
		};
	}
);

// Helper function to get time since last ping
export function getTimeSincePing(connectionState: ConnectionState): number | null {
	if (!connectionState.lastPing) return null;
	return new Date().getTime() - connectionState.lastPing.getTime();
}

// Helper function to check if connection is fresh (less than 30 seconds)
export function isConnectionFresh(connectionState: ConnectionState): boolean {
	const timeSince = getTimeSincePing(connectionState);
	return timeSince !== null && timeSince < 30000;
}