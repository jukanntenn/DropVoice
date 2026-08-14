/**
 * Tauri IPC mock (spec 06 §2.4).
 *
 * Re-exports a `vi.fn()` for `@tauri-apps/api/core::invoke` so component tests
 * can assert on calls without touching the real bridge. The actual
 * `vi.mock('@tauri-apps/api/core', ...)` call belongs at the top of
 * `tests/setup.ts` (vi.mock must be top-level; re-exporting just the fn here
 * keeps the mock definition and the assertion handle in sync).
 */
import { vi } from 'vitest';

export const mockInvoke = vi.fn();
