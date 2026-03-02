# Multi-Device Connection UX Improvements Design

**Date**: 2026-03-03
**Status**: Design Approved
**Author**: AI Assistant

---

## Overview

Fix three user experience issues in the multi-device connection feature:

1. **First scan auto-connect** - Restore the "plug-and-play" experience from single-device version
2. **Fix QR scan white screen** - Resolve camera initialization issue
3. **Optimize URL input** - Support direct HTTP address paste (matching PC display format)

**Scope**: Frontend only (`src/mobile/`), no backend modifications

---

## Problem 1: First Scan Auto-Connect

### Current Behavior

```
User scans QR code → Opens http://192.168.1.100:38425
    ↓
React app loads
    ↓
Reads from localStorage → Empty
    ↓
Shows: "No devices, please add"
    ↓
User must click "Add" → Scan again → Connected
```

**Problem**: Extra step compared to original single-device version

### Expected Behavior

```
User scans QR code → Opens http://192.168.1.100:38425
    ↓
React app loads
    ↓
Detects: localStorage empty + valid window.location.host
    ↓
Automatically adds current PC device
    ↓
Shows: Device card with green dot (connected)
```

### Implementation

**File**: `src/mobile/hooks/useDeviceManager.ts`

```typescript
useEffect(() => {
  const storage = loadDeviceStorage();
  const loadedDevices = storage.devices.map(storedToDevice);

  // Auto-detect and add first device
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
      loadedDevices.push(autoDevice);
      saveDeviceStorage({
        devices: [deviceToStored(autoDevice)],
        lastActiveDeviceId: autoDevice.id
      });
    }
  }

  setDevices(loadedDevices);
  // ... rest of initialization
}, []);
```

### Data Flow

```
Page Load
    ↓
useDeviceManager initialization
    ↓
Check localStorage
    ↓
Empty? → Yes → Check window.location.host
    ↓              ↓
    No          Has host? → Yes → Auto-create device → Save → Connect
                      ↓
                     No → Show empty state
```

---

## Problem 2: QR Scan White Screen Fix

### Root Cause Analysis

**Problem**: `Html5Qrcode` library initializes before DOM element is fully rendered.

**Causes**:
1. `useEffect` executes immediately after component render
2. DOM element `#qr-scanner` may not be inserted into document yet
3. `Html5Qrcode` cannot find element, throws error

### Solution

**File**: `src/mobile/components/QRScanner.tsx`

**Strategy**:
1. Use `useRef` to track scanner instance, avoid duplicates
2. Use `setTimeout` to delay initialization, ensure DOM ready
3. Add error boundary and cleanup logic

```typescript
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
    }, 100); // 100ms delay

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

**Key Improvements**:
- ✅ Delay 100ms for DOM readiness
- ✅ Ref tracking prevents memory leaks
- ✅ Filter "No barcode" noise errors
- ✅ Clearer error messages

---

## Problem 3: URL Input Optimization

### User Expectation

**Current**:
- Placeholder: `ws://192.168.1.xxx:38425/ws`
- User must manually input WebSocket URL

**Expected**:
- Placeholder: `http://192.168.1.100:38425`
- User pastes HTTP address directly from PC display
- System auto-converts to WebSocket URL

### Implementation

**File**: `src/mobile/components/AddDeviceModal.tsx`

**Changes**:

1. **Update placeholder text**:
```typescript
// Before
placeholder={t("devices.urlPlaceholder")}  // "ws://192.168.1.xxx:38425/ws"

// After
placeholder="http://192.168.1.100:38425"
```

2. **Show conversion preview**:
```typescript
const normalizedUrl = useMemo(() => {
  if (!urlInput.trim()) return null;
  const wsUrl = normalizeToWebSocketUrl(urlInput);
  return wsUrl;
}, [urlInput]);

// Display below input:
{normalizedUrl && normalizedUrl !== urlInput && (
  <div className="text-xs text-slate-500 dark:text-slate-400 mt-1">
    Will connect to: {normalizedUrl}
  </div>
)}
```

3. **Update i18n translations**:
```json
{
  "devices": {
    "urlPlaceholder": "http://192.168.1.100:38425",
    "urlHint": "Paste the address shown on PC"
  }
}
```

### User Flow

```
1. User sees on PC: http://192.168.1.100:38425
2. User copies address
3. User pastes on mobile
4. System shows: "Will connect to: ws://192.168.1.100:38425/ws"
5. User clicks "Add Device"
```

---

## Test Plan

### Problem 1: First Scan Auto-Connect

| Scenario | Expected Result |
|----------|----------------|
| New user scans QR code | Auto-connects, device card shows green |
| Refresh page | Device persists, auto-reconnects |
| Clear localStorage | Re-adds current PC automatically |
| file:// protocol access | No auto-add (avoid non-HTTP environment) |

### Problem 2: QR Scan White Screen

| Scenario | Expected Result |
|----------|----------------|
| Click "Start Scan" | Camera starts, viewfinder displayed |
| Scan valid QR code | Auto-recognize and add device |
| Camera permission denied | Show error: "Camera init failed" |
| Rapid stop/start toggle | No memory leaks, works correctly |

### Problem 3: URL Input Optimization

| Scenario | Expected Result |
|----------|----------------|
| Input HTTP URL | Show conversion hint, connect to correct WebSocket |
| Input HTTPS URL | Convert to WSS, connect successfully |
| Input WebSocket URL | Use directly, no conversion needed |
| Input invalid URL | Show error: "Invalid connection address" |

---

## Implementation Checklist

**File Modifications**:
- [ ] `src/mobile/hooks/useDeviceManager.ts` - Add auto-detection logic
- [ ] `src/mobile/components/QRScanner.tsx` - Fix white screen issue
- [ ] `src/mobile/components/AddDeviceModal.tsx` - Optimize URL input
- [ ] `src/mobile/locales/zh.json` - Update translations
- [ ] `src/mobile/locales/en.json` - Update translations

**Testing**:
- [ ] Real device QR scan test
- [ ] Simulator URL input test
- [ ] Edge cases (permission denied, invalid URL, etc.)

---

## Summary

**Core Improvements**:
1. **First scan connects immediately** - Restore plug-and-play experience
2. **Fix QR scanning** - Resolve white screen, enable normal scan-to-add
3. **Simplify URL input** - Direct HTTP paste from PC display

**No Backend Changes** - All improvements are mobile frontend only

**Backward Compatible** - Does not affect existing multi-device users
