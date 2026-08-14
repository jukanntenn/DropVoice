import { QueryClient } from '@tanstack/react-query';

/**
 * Shared React Query client configured per spec 02 section 2.3.
 *
 * - 5 minute stale time so server settings don't refetch on every focus
 * - 3 retries for transient network blips
 * - No refetch on window focus (we use explicit polling for connection info)
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 3,
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: 0,
    },
  },
});

export const QUERY_KEYS = {
  connectionInfo: ['connection-info'] as const,
  settings: ['settings'] as const,
  devices: ['devices'] as const,
  health: ['health'] as const,
} as const;
