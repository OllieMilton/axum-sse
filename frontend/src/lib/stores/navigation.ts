// Navigation utilities using SvelteKit's built-in routing
import { derived, type Readable } from 'svelte/store';
import { page } from '$app/stores';

export interface NavigationInfo {
	isMainPage: boolean;
	isAboutPage: boolean;
	pageDisplayName: string;
	pageTitle: string;
}

// Helper function to get page information
function getPageInfo(path: string): { title: string; displayName: string } {
	switch (path) {
		case '/':
			return { title: 'Axum SSE Demo - Main', displayName: 'Main' };
		case '/about':
			return { title: 'Axum SSE Demo - About', displayName: 'About' };
		default:
			return { title: 'Axum SSE Demo', displayName: 'Unknown' };
	}
}

// Derived store for navigation info with computed properties based on SvelteKit's page store
export const navigationInfo: Readable<NavigationInfo> = derived(
	page,
	($page) => {
		const currentPath = $page.url.pathname;
		const isMainPage = currentPath === '/';
		const isAboutPage = currentPath === '/about';
		
		const pageInfo = getPageInfo(currentPath);
		
		return {
			isMainPage,
			isAboutPage,
			pageDisplayName: pageInfo.displayName,
			pageTitle: pageInfo.title
		};
	}
);

// Helper function to check if a path is the current page
export function isCurrentPage(path: string, currentPath: string): boolean {
	return currentPath === path;
}

// Helper function to get page title
export function getPageTitle(path: string): string {
	return getPageInfo(path).title;
}