# Fix: Auto-switch on Scan & Storage Resilience

## Issues

- **#48**: Scanning a QR code for an already-saved device doesn't switch to it — `addDevice()` rejects duplicates silently.
- **#49**: Draft persistence breaks in some browsers (e.g., WeChat WebView) because localStorage is the only backend, failures are silent, and no lifecycle-triggered saves exist.

## Task 1: Create unified storage utility

Create `src/mobile/utils/createStorage.ts` — a shared abstraction with fallback chain.

```
createStorage(prefix: string) → { get, set, remove }
```

**Fallback chain**: try localStorage → sessionStorage → in-memory `Map`.

- `get(key)`: returns value from first available backend, or `null`
- `set(key, value)`: writes to all available backends (keeps them in sync), returns `boolean` (false if all failed)
- `remove(key)`: removes from all available backends

Key format: `{prefix}_{host}_{deviceId}` (same as current pattern).

## Task 2: Migrate storage.ts to use createStorage

Replace direct `localStorage` calls in `src/mobile/utils/storage.ts` with `createStorage("dropvoice")` instance.

- `saveDraft`, `loadDraft`, `clearDraft` — use the storage instance
- `saveLastSent`, `loadLastSent`, `clearLastSent`, `hasLastSent` — same
- Remove manual `try/catch` blocks (handled by createStorage)

## Task 3: Migrate deviceStorage.ts to use createStorage

Replace direct `localStorage` calls in `src/mobile/utils/deviceStorage.ts` with `createStorage("dropvoice")` instance.

- `loadDeviceStorage`, `saveDeviceStorage` — use the storage instance
- Key: `dropvoice:devices:v1` (unchanged)

## Task 4: Migrate i18n.ts to use createStorage

Replace direct `localStorage` calls in `src/mobile/i18n.ts` with the same storage instance.

## Task 5: Add lifecycle-triggered draft saves in App.tsx

In `src/mobile/App.tsx`, add two event handlers:

1. **`visibilitychange`**: when `document.visibilityState === "hidden"`, immediately save current draft (no debounce).
2. **`beforeunload`**: save current draft synchronously.

This catches WeChat destroying the WebView while backgrounded, and page close/refresh.

## Task 6: Surface storage errors via toast

In `createStorage`, track whether any backend is functional. If all backends fail on first write attempt, expose a flag.

In `App.tsx` (or a small hook), check this flag on mount and show a warning toast: "Storage unavailable — drafts won't persist this session."

## Task 7: Auto-switch to existing device on QR scan

Modify `src/mobile/hooks/useMultiWebSocket.ts` — change `addDevice()`:

- When URL matches an existing device (the `devicesRef.current.some(d => d.url === url)` check):
  - Find the existing device
  - Update its `lastConnected` timestamp
  - Sort it to front (by calling `setDevices` with updated array)
  - Call `setActiveDeviceId(existingDevice.id)`
  - Reset connection state: if disconnected/exhausted retries, reconnect
  - Return `true` (not `false`)

## Task 8: Close modal and show success toast on existing-device switch

Modify `src/mobile/components/AddDeviceModal.tsx` — in `onScan`:

- Since `addDevice` now returns `true` for existing devices, the existing "added" overlay and close logic will fire naturally
- No special handling needed — the `onAdd` callback returning `true` triggers `close()` in the parent

## Task 9: Update error key handling

The `lastError` watcher in `App.tsx` shows toasts for `devices.*` errors. After the fix, `devices.alreadyExists` is no longer an error — it's a success path. Verify no stale error keys leak through.

## Files to modify

| File | Change |
|------|--------|
| `src/mobile/utils/createStorage.ts` | **New** — unified storage utility |
| `src/mobile/utils/storage.ts` | Migrate to createStorage |
| `src/mobile/utils/deviceStorage.ts` | Migrate to createStorage |
| `src/mobile/i18n.ts` | Migrate to createStorage |
| `src/mobile/App.tsx` | Lifecycle saves + storage error toast |
| `src/mobile/hooks/useMultiWebSocket.ts` | Auto-switch on duplicate URL scan |

## Verification

1. **Issue #48**: Open mobile page with 2+ saved devices. Scan QR of an already-saved device → it becomes active, modal closes, WebSocket connects.
2. **Issue #49 (localStorage works)**: Type text, switch apps, return → draft persists. Close page, reopen → draft persists.
3. **Issue #49 (localStorage blocked)**: In browser devtools, override `localStorage.setItem` to throw. Type text → warning toast appears. Text still persists within session (sessionStorage/in-memory). Reload → text gone (expected).
4. **Regression**: Normal flow — add new device via QR, send text, restore last sent, swipe between devices — all still work.
