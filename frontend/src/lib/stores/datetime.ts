// Persistent datetime store for SPA navigation
import { writable, type Writable } from 'svelte/store';

// Check if we're in the browser (client-side)
const browser = typeof window !== 'undefined';

export interface DateTimeState {
	currentTime: string | null;
	lastUpdated: Date | null;
	timezone: string;
	format: string;
}

// Storage key for localStorage persistence
const STORAGE_KEY = 'axum-sse-datetime';

// Create a persistent store that saves to localStorage
function createPersistentDateTimeStore(): Writable<DateTimeState> & {
	updateTime: (timeString: string) => void;
	clearTime: () => void;
	getStoredState: () => DateTimeState | null;
} {
	// Initial state
	const initialState: DateTimeState = {
		currentTime: null,
		lastUpdated: null,
		timezone: 'Europe/London',
		format: 'DD/MM/YYYY HH:mm:ss'
	};

	// Load from localStorage if available
	function loadFromStorage(): DateTimeState {
		if (!browser) return initialState;
		
		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored) {
				const parsed = JSON.parse(stored);
				return {
					...initialState,
					...parsed,
					lastUpdated: parsed.lastUpdated ? new Date(parsed.lastUpdated) : null
				};
			}
		} catch (error) {
			console.warn('Failed to load datetime from localStorage:', error);
		}
		
		return initialState;
	}

	// Save to localStorage
	function saveToStorage(state: DateTimeState) {
		if (!browser) return;
		
		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify({
				...state,
				lastUpdated: state.lastUpdated?.toISOString()
			}));
		} catch (error) {
			console.warn('Failed to save datetime to localStorage:', error);
		}
	}

	// Create the writable store with initial state from storage
	const { subscribe, set, update } = writable<DateTimeState>(loadFromStorage());

	// Subscribe to changes and save to storage
	if (browser) {
		subscribe((state) => {
			saveToStorage(state);
		});
	}

	return {
		subscribe,
		set,
		update,

		// Update the current time
		updateTime: (timeString: string) => {
			update(state => ({
				...state,
				currentTime: timeString,
				lastUpdated: new Date()
			}));
		},

		// Clear the stored time
		clearTime: () => {
			update(state => ({
				...state,
				currentTime: null,
				lastUpdated: null
			}));
		},

		// Get the current stored state
		getStoredState: () => {
			if (!browser) return null;
			
			try {
				const stored = localStorage.getItem(STORAGE_KEY);
				return stored ? JSON.parse(stored) : null;
			} catch (error) {
				console.warn('Failed to read stored datetime state:', error);
				return null;
			}
		}
	};
}

// Export the datetime store instance
export const dateTimeStore = createPersistentDateTimeStore();

// Helper function to check if stored time is stale (older than 2 minutes)
export function isStoredTimeStale(state: DateTimeState): boolean {
	if (!state.lastUpdated) return true;
	
	const now = new Date();
	const twoMinutes = 2 * 60 * 1000; // 2 minutes in milliseconds
	
	return (now.getTime() - state.lastUpdated.getTime()) > twoMinutes;
}

// Helper function to format relative time
export function getRelativeTimeString(lastUpdated: Date | null): string {
	if (!lastUpdated) return 'Never updated';
	
	const now = new Date();
	const diffMs = now.getTime() - lastUpdated.getTime();
	const diffSeconds = Math.floor(diffMs / 1000);
	const diffMinutes = Math.floor(diffSeconds / 60);
	
	if (diffSeconds < 30) {
		return 'Just now';
	} else if (diffSeconds < 60) {
		return `${diffSeconds} seconds ago`;
	} else if (diffMinutes < 60) {
		return `${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`;
	} else {
		return lastUpdated.toLocaleTimeString();
	}
}

// Helper function to check if we should show a stale data warning
export function shouldShowStaleWarning(state: DateTimeState): boolean {
	return state.currentTime !== null && isStoredTimeStale(state);
}