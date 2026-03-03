# Multi-Device UX Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix three UX issues in multi-device connection: auto-connect on first scan, fix QR scanner white screen, and optimize URL input for HTTP paste.

**Architecture:** Mobile frontend only changes. Auto-detect first device in useDeviceManager hook, fix QR scanner initialization timing, and update AddDeviceModal to show HTTP URLs with WebSocket conversion preview.

**Tech Stack:** React 18, TypeScript, html5-qrcode, localStorage

---

## Task 1: Add Auto-Detect First Device Logic

**Files:**
- Modify: `src/mobile/hooks/useDeviceManager.ts`

**Step 1: Add helper function to extract device name from host**

Add to `src/mobile/hooks/useDeviceManager.ts`:

```typescript
import { useCallback, useEffect, useMemo, useState } from "react";
import type { Device } from "../types";
import {
  deviceToStored,
  generateDeviceId,
  loadDeviceStorage,
  saveDeviceStorage,
  storedToDevice,
} from "../utils/deviceStorage";

function getDefaultDeviceNameFromHost(host: string): string {
  try {
    const hostname = host.split(':')[0]; // Remove port
    if (/^\d+\.\d+\.\d+\.\d+$/.test(hostname)) {
      const lastOctet = hostname.split('.').pop();
      return `PC-${lastOctet ?? hostname}`;
    }
    return hostname ? `PC-${hostname}` : "Unknown Device";
  } catch {
    return "Unknown Device";
  }
}
```

**Step 2: Modify initialization useEffect to auto-detect first device**

Replace the existing `useEffect` in `useDeviceManager`:

```typescript
useEffect(() => {
  const storage = loadDeviceStorage();
  let loadedDevices = storage.devices.map(storedToDevice);

  // Auto-detect and add first device if storage is empty
  if (loadedDevices.length === 0) {
    const currentHost = window.location.host;
    if (currentHost && !window.location.protocol.startsWith('file')) {
      const autoDevice: Device = {
        id: generateDeviceId(),
        name: getDefaultDeviceNameFromHost(currentHost),
        url: `ws://${currentHost}/ws`,
        status: 'disconnected',
        lastConnected: Date.now()
      };
      loadedDevices = [autoDevice];
      saveDeviceStorage({
        devices: [deviceToStored(autoDevice)],
        lastActiveDeviceId: autoDevice.id
      });
    }
  }

  setDevices(loadedDevices);

  const hasStoredActive =
    typeof storage.lastActiveDeviceId === "string" &&
    loadedDevices.some((d) => d.id === storage.lastActiveDeviceId);

  setActiveDeviceIdState(
    hasStoredActive
      ? storage.lastActiveDeviceId
      : loadedDevices.length > 0
        ? loadedDevices[0].id
        : null,
  );

  setIsInitialized(true);
}, []);
```

**Step 3: Test auto-detection locally**

Run: `pnpm tauri dev`
Steps:
1. Clear browser localStorage: DevTools → Application → Local Storage → Clear all
2. Navigate to mobile page (should show one device card with PC-{IP})
3. Refresh page (device should persist)
4. Check localStorage contains device entry

Expected: Device automatically created and connected

**Step 4: Commit**

```bash
git add src/mobile/hooks/useDeviceManager.ts
git commit -m "feat(mobile): auto-detect and add first device on initial load

- Detects empty localStorage on page load
- Automatically adds current PC as first device
- Generates device name from host (PC-{last-octet})
- Preserves plug-and-play experience from single-device version"
```

---

## Task 2: Fix QR Scanner White Screen Issue

**Files:**
- Modify: `src/mobile/components/QRScanner.tsx`

**Step 1: Add refs for scanner instance and initialization timer**

Replace entire `QRScanner.tsx` content:

```typescript
import { useEffect, useRef } from "react";
import { Html5Qrcode } from "html5-qrcode";

interface QRScannerProps {
  onScan: (text: string) => void;
  onError?: (error: string) => void;
}

export function QRScanner({ onScan, onError }: QRScannerProps) {
  const scannerRef = useRef<Html5Qrcode | null>(null);
  const isStoppedRef = useRef(false);
  const initTimerRef = useRef<number | null>(null);

  useEffect(() => {
    isStoppedRef.current = false;

    // Delay initialization to ensure DOM is ready
    initTimerRef.current = window.setTimeout(() => {
      if (isStoppedRef.current) return;

      try {
        const scanner = new Html5Qrcode("qr-scanner");
        scannerRef.current = scanner;

        scanner.start(
          { facingMode: "environment" },
          { fps: 10, qrbox: { width: 250, height: 250 } },
          (decodedText) => {
            if (isStoppedRef.current) return;
            isStoppedRef.current = true;
            onScan(decodedText);
            scanner.stop().catch(() => {});
          },
          (errorMessage) => {
            // Silent handling of scan noise errors
            if (!errorMessage.includes('No barcode')) {
              onError?.(errorMessage);
            }
          }
        ).catch((err) => {
          onError?.(`Camera init failed: ${err.message || err}`);
        });
      } catch (err) {
        onError?.(`Scanner init failed: ${err instanceof Error ? err.message : String(err)}`);
      }
    }, 100); // 100ms delay for DOM readiness

    return () => {
      isStoppedRef.current = true;
      if (initTimerRef.current !== null) {
        clearTimeout(initTimerRef.current);
      }
      scannerRef.current?.stop().catch(() => {
        scannerRef.current = null;
      });
    };
  }, [onError, onScan]);

  return (
    <div
      id="qr-scanner"
      className="w-full aspect-square bg-black rounded-2xl overflow-hidden"
    />
  );
}
```

**Step 2: Test QR scanner on mobile device**

Run: `pnpm tauri dev`

Test on real mobile device:
1. Access mobile page via LAN
2. Click "Add Device" → "Scan QR"
3. Click "Start Scan"
4. Verify camera viewfinder appears (no white screen)
5. Scan a QR code containing a URL
6. Verify device is added

Expected: Camera starts correctly, QR code is recognized

**Step 3: Test edge cases**

1. Deny camera permission → Should show error message
2. Rapid click "Stop/Start" → Should work without crashes
3. Scan during scan → Should handle gracefully

**Step 4: Commit**

```bash
git add src/mobile/components/QRScanner.tsx
git commit -m "fix(mobile): resolve QR scanner white screen issue

- Add 100ms delay for DOM readiness before initialization
- Use refs to track scanner instance and prevent duplicates
- Filter out 'No barcode' noise errors
- Proper cleanup on unmount to prevent memory leaks"
```

---

## Task 3: Optimize URL Input for HTTP Paste

**Files:**
- Modify: `src/mobile/components/AddDeviceModal.tsx`
- Modify: `src/mobile/locales/zh.json`
- Modify: `src/mobile/locales/en.json`

**Step 1: Update AddDeviceModal to show conversion preview**

Modify the URL input section in `AddDeviceModal.tsx`:

Find the URL input section and add the conversion hint:

```typescript
// Inside the component, after the existing useMemo for normalizedUrl:

// Show conversion preview
{normalizedUrl && normalizedUrl !== urlInput && (
  <div className="text-xs text-slate-500 dark:text-slate-400 mt-1 px-1">
    将连接到: {normalizedUrl}
  </div>
)}
```

Also update the placeholder text in the input element:

```typescript
<input
  type="text"
  value={urlInput}
  onChange={(e) => setUrlInput(e.target.value)}
  placeholder="http://192.168.1.100:38425"
  className="w-full rounded-2xl border border-slate-300 bg-white px-4 py-3 text-slate-900 outline-none focus:ring-2 focus:ring-primary/50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
/>
```

**Step 2: Update Chinese translations**

Modify `src/mobile/locales/zh.json`:

```json
{
  "devices": {
    "urlPlaceholder": "http://192.168.1.100:38425",
    "willConnectTo": "将连接到: {{url}}"
  }
}
```

**Step 3: Update English translations**

Modify `src/mobile/locales/en.json`:

```json
{
  "devices": {
    "urlPlaceholder": "http://192.168.1.100:38425",
    "willConnectTo": "Will connect to: {{url}}"
  }
}
```

**Step 4: Test URL input conversion**

Run: `pnpm tauri dev`

Test cases:
1. Paste `http://192.168.1.100:38425` → Should show "将连接到: ws://192.168.1.100:38425/ws"
2. Paste `https://example.com` → Should show "将连接到: wss://example.com/ws"
3. Paste `ws://192.168.1.100:38425/ws` → Should not show conversion hint
4. Enter invalid URL → Should show error on submit

**Step 5: Commit**

```bash
git add src/mobile/components/AddDeviceModal.tsx src/mobile/locales/zh.json src/mobile/locales/en.json
git commit -m "feat(mobile): optimize URL input for direct HTTP paste

- Change placeholder to HTTP format matching PC display
- Show WebSocket conversion preview when HTTP URL is entered
- Update i18n strings for new hint text
- Simplify user workflow: copy from PC, paste on mobile"
```

---

## Task 4: Integration Testing

**Files:**
- No file modifications

**Step 1: Test complete user flow**

Run: `pnpm tauri dev`

**Scenario A: First-time user (auto-connect)**
1. Use fresh browser/incognito window
2. Scan QR code from PC
3. Verify: Device card appears with green dot immediately
4. Type text and send → Verify text appears on PC
5. Refresh page → Verify device persists and reconnects

**Scenario B: Add second device via QR**
1. Click "Add" button
2. Click "Scan QR" tab
3. Click "Start Scan" → Verify camera works
4. Scan QR code from another PC
5. Verify: Second device card appears
6. Click device cards to switch → Verify active selection

**Scenario C: Add device via URL input**
1. Click "Add" button
2. Click "Input URL" tab
3. Copy URL from PC display (http://...)
4. Paste into input field
5. Verify: Shows "将连接到: ws://..."
6. Click "Add Device"
7. Verify: Device card appears and connects

**Step 2: Test error scenarios**

1. Test with invalid URL → Should show error
2. Test with duplicate URL → Should show "already exists"
3. Test with 6th device → Should show "max 5 devices"
4. Test network disconnect → Device should show disconnected, auto-reconnect

**Step 3: Verify localStorage persistence**

1. Add multiple devices
2. Close browser
3. Reopen mobile page
4. Verify all devices restored with correct names

**Step 4: Create summary commit**

```bash
git add .
git commit -m "feat(mobile): complete multi-device UX improvements

All three improvements implemented and tested:
- Auto-connect on first scan (plug-and-play experience)
- Fixed QR scanner white screen issue
- Optimized URL input for HTTP paste

Tested on real device, all scenarios passing."
```

---

## Post-Implementation Tasks

### Final Verification Checklist

- [ ] First-time scan automatically adds and connects device
- [ ] QR scanner starts without white screen
- [ ] URL input accepts HTTP addresses with conversion preview
- [ ] All devices persist after page refresh
- [ ] Device switching works correctly
- [ ] Error messages display appropriately
- [ ] localStorage contains correct data format

### Build Production Bundle

```bash
pnpm build
```

Verify no TypeScript errors and bundle size is acceptable.

### Documentation Updates

Update any relevant documentation if behavior has changed from original design.

---

## Summary

This plan implements three UX improvements in 4 bite-sized tasks:

1. **Task 1**: Add auto-detection logic in useDeviceManager hook
2. **Task 2**: Fix QR scanner with delayed initialization
3. **Task 3**: Update AddDeviceModal for HTTP URL input
4. **Task 4**: Integration testing of complete user flow

**Estimated completion time**: 1-2 hours

**No backend changes required** - All improvements are mobile frontend only.
