# Camera Support Detection and Fallback Design

**Date**: 2026-03-03
**Status**: Design Approved
**Author**: AI Assistant

---

## Overview

Fix the QR scanner white screen issue by detecting camera support and gracefully falling back to manual input when camera is not available due to HTTP restrictions or browser limitations.

**Problem**: Users on HTTP (non-HTTPS) connections experience a white screen when trying to scan QR codes, with no error message or guidance.

**Solution**: Detect camera support before attempting to initialize the scanner, show user-friendly error messages, and automatically switch to manual input when camera is unavailable.

**Scope**: Frontend only (`src/mobile/`), no backend modifications

---

## Problem Analysis

### Current Behavior

```
User clicks "Start Scan"
    ↓
isScanning = true, QRScanner component renders
    ↓
useEffect executes, starts retry loop
    ↓
Html5Qrcode.start() called
    ↓
Browser blocks camera access (HTTP restriction)
    ↓
Error caught but not properly displayed
    ↓
Result: White screen, no feedback
```

### Root Cause

1. **Browser Security Policy**: Modern mobile browsers (Chrome 86+, iOS Safari 14.3+) require HTTPS for camera access, except for `localhost`
2. **Silent Failure**: `html5-qrcode` library's error handling doesn't provide clear feedback to users
3. **No Fallback**: Application doesn't detect camera support before attempting to use it
4. **Poor UX**: Users see white screen with no explanation or alternative

### Browser Requirements

| Browser | Camera Access Requirement |
|---------|--------------------------|
| iOS Safari 14.3+ | HTTPS required (no exceptions for LAN IP) |
| iOS Edge | HTTPS required (uses Safari WebView) |
| Android Chrome 86+ | HTTPS required |
| Android Edge | HTTPS required (uses Chrome WebView) |
| Older browsers | May allow HTTP on LAN |

**Exception**: `localhost` is treated as a secure context, but this doesn't apply to LAN IP addresses.

---

## Solution Design

### Core Strategy: Detect → Inform → Fallback

```
User clicks "Start Scan"
    ↓
Immediately check camera support
    ↓
    ├─ Camera available → Show scanner
    │
    └─ Camera unavailable → Show friendly error
                           ↓
                       Auto-switch to manual input (2s delay)
                           ↓
                       User can manually enter URL
```

---

## Implementation Details

### 1. Camera Support Detection Utility

**File**: `src/mobile/utils/cameraSupport.ts` (new file)

**Purpose**: Centralized camera support detection logic

**API**:
```typescript
interface CameraSupportResult {
  supported: boolean;
  reason?: string;
  userMessage?: string;
}

async function checkCameraSupport(): Promise<CameraSupportResult>
```

**Detection Logic**:

```typescript
export async function checkCameraSupport(): Promise<CameraSupportResult> {
  // 1. Check if MediaDevices API exists
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    return {
      supported: false,
      reason: "MediaDevices API not available",
      userMessage: "您的浏览器不支持摄像头功能",
    };
  }

  // 2. Check secure context (HTTPS or localhost)
  const isSecureContext = window.isSecureContext;
  const isLocalhost = window.location.hostname === "localhost" ||
                      window.location.hostname === "127.0.0.1";

  if (!isSecureContext && !isLocalhost) {
    return {
      supported: false,
      reason: "Not a secure context (requires HTTPS or localhost)",
      userMessage: "摄像头需要 HTTPS 或 localhost 才能使用。已自动切换到手动输入。",
    };
  }

  // 3. Try to actually access camera
  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: "environment" }
    });

    // Immediately stop the stream (we just wanted to test access)
    stream.getTracks().forEach(track => track.stop());

    return { supported: true };
  } catch (err) {
    const error = err as Error;

    if (error.name === "NotAllowedError") {
      return {
        supported: false,
        reason: "Permission denied",
        userMessage: "摄像头权限被拒绝。请在浏览器设置中允许访问摄像头。",
      };
    }

    if (error.name === "NotFoundError") {
      return {
        supported: false,
        reason: "No camera found",
        userMessage: "未找到摄像头设备。",
      };
    }

    return {
      supported: false,
      reason: error.message,
      userMessage: `摄像头初始化失败: ${error.message}`,
    };
  }
}
```

**Key Points**:
- ✅ Checks API availability first (fast fail)
- ✅ Checks secure context before requesting permission
- ✅ Actually tests camera access (catches permission denials)
- ✅ Returns user-friendly messages
- ✅ Cleans up test stream immediately

---

### 2. AddDeviceModal Component Improvements

**File**: `src/mobile/components/AddDeviceModal.tsx`

**Changes**:

#### A. Import camera support utility

```typescript
import { checkCameraSupport } from "../utils/cameraSupport";
```

#### B. Replace start scan button handler

**Current**:
```typescript
onClick={() => setIsScanning((v) => !v)}
```

**New**:
```typescript
onClick={async () => {
  if (isScanning) {
    // Stop scanning
    setIsScanning(false);
    return;
  }

  // Start scanning - check camera support first
  setIsScanning(true);
  setError(null);

  const result = await checkCameraSupport();

  if (!result.supported) {
    // Show error
    setError(result.userMessage || result.reason);

    // Auto-switch to manual input after 2 seconds
    setTimeout(() => {
      setTab("input");
      setIsScanning(false);
      setError(null);
    }, 2000);

    return;
  }

  // Camera is available, scanner will initialize
}}
```

#### C. Add hint banner in scan tab

**Location**: Inside the scan tab, above the scanner/placeholder

```typescript
{tab === "scan" && (
  <div className="mb-4 rounded-2xl bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:bg-amber-900/30 dark:text-amber-200">
    <div className="flex items-start gap-2">
      <svg className="h-5 w-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
        <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
      </svg>
      <div>
        <p className="font-medium">提示</p>
        <p className="mt-1">摄像头需要 HTTPS 或 localhost 才能使用。如无法扫描，请使用手动输入。</p>
      </div>
    </div>
  </div>
)}
```

#### D. Improve error display

**Current**:
```typescript
{error ? (
  <div className="mt-4 rounded-2xl bg-red-100 px-4 py-3 text-sm text-red-700 dark:bg-red-900/30 dark:text-red-200">
    {error}
  </div>
) : null}
```

**New**:
```typescript
{error && (
  <div className="mt-4 rounded-2xl bg-red-100 px-4 py-3 text-sm text-red-700 dark:bg-red-900/30 dark:text-red-200">
    <div className="flex items-start gap-2">
      <svg className="h-5 w-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
      </svg>
      <div>
        <p className="font-medium">无法访问摄像头</p>
        <p className="mt-1">{error}</p>
      </div>
    </div>
  </div>
)}
```

---

### 3. QRScanner Component Improvements

**File**: `src/mobile/components/QRScanner.tsx`

**Changes**:

#### A. Add loading state

```typescript
const [isLoading, setIsLoading] = useState(true);
```

#### B. Update scanner initialization

```typescript
scanner
  .start(
    { facingMode: "environment" },
    { fps: 10, qrbox: { width: 250, height: 250 } },
    (decodedText) => {
      if (isStoppedRef.current) return;
      isStoppedRef.current = true;
      setIsLoading(false);
      onScan(decodedText);
      scanner.stop().catch(() => {});
    },
    (errorMessage) => {
      const msg = String(errorMessage);
      if (msg.includes("No barcode") || msg.includes("NotFoundException")) return;
      onError?.(msg);
    },
  )
  .then(() => {
    setIsLoading(false);
  })
  .catch((err) => {
    setIsLoading(false);
    const msg =
      err && typeof err === "object" && "message" in err
        ? String(err.message)
        : String(err);

    // Provide more specific error message
    if (msg.includes("NotAllowedError") || msg.includes("Permission denied")) {
      onError?.("摄像头权限被拒绝。请在浏览器设置中允许访问摄像头。");
    } else if (msg.includes("NotFoundError")) {
      onError?.("未找到摄像头设备。");
    } else if (msg.includes("NotReadableError")) {
      onError?.("摄像头被其他应用程序占用。");
    } else {
      onError?.(`摄像头初始化失败: ${msg}`);
    }
  });
```

#### C. Update render to show loading state

```typescript
return (
  <div className="relative w-full aspect-square bg-black rounded-2xl overflow-hidden">
    {isLoading && (
      <div className="absolute inset-0 flex items-center justify-center bg-slate-900 z-10">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-white border-t-transparent mx-auto" />
          <p className="mt-3 text-sm text-white">正在启动摄像头...</p>
        </div>
      </div>
    )}
    <div id="qr-scanner" className="w-full h-full" />
  </div>
);
```

---

### 4. Internationalization Updates

**File**: `src/mobile/i18n.ts`

**Add new translation keys**:

```typescript
// Chinese translations
const zh = {
  // ... existing translations
  devices: {
    // ... existing device translations
    cameraRequiresHttps: "摄像头需要 HTTPS 或 localhost 才能使用",
    switchedToManualInput: "已自动切换到手动输入",
    cameraPermissionDenied: "摄像头权限被拒绝。请在浏览器设置中允许访问摄像头。",
    cameraNotSupported: "您的浏览器不支持摄像头功能",
    cameraNotFound: "未找到摄像头设备。",
    cameraInUse: "摄像头被其他应用程序占用。",
    startingCamera: "正在启动摄像头...",
    cameraInitFailed: "摄像头初始化失败",
    scanHintHttps: "摄像头需要 HTTPS 或 localhost 才能使用。如无法扫描，请使用手动输入。",
  },
};

// English translations
const en = {
  // ... existing translations
  devices: {
    // ... existing device translations
    cameraRequiresHttps: "Camera requires HTTPS or localhost",
    switchedToManualInput: "Switched to manual input",
    cameraPermissionDenied: "Camera permission denied. Please allow camera access in browser settings.",
    cameraNotSupported: "Your browser doesn't support camera",
    cameraNotFound: "No camera device found.",
    cameraInUse: "Camera is being used by another application.",
    startingCamera: "Starting camera...",
    cameraInitFailed: "Camera initialization failed",
    scanHintHttps: "Camera requires HTTPS or localhost. Please use manual input if scanning is unavailable.",
  },
};
```

---

## User Experience Flow

### Scenario 1: HTTP Connection (Camera Unavailable)

```
1. User opens "Add Device" modal
   ↓
2. "Scan QR" tab shows hint: "Camera requires HTTPS..."
   ↓
3. User clicks "Start Scan"
   ↓
4. Camera support check runs
   ↓
5. Error displayed: "Camera requires HTTPS or localhost. Switched to manual input."
   ↓
6. After 2 seconds, auto-switches to "Manual Input" tab
   ↓
7. User manually enters URL
```

### Scenario 2: HTTPS/Localhost (Camera Available)

```
1. User opens "Add Device" modal
   ↓
2. User clicks "Start Scan"
   ↓
3. Camera support check passes
   ↓
4. Scanner initializes with loading indicator
   ↓
5. Camera viewfinder appears
   ↓
6. User scans QR code
```

### Scenario 3: Permission Denied

```
1. User clicks "Start Scan"
   ↓
2. Camera support check runs
   ↓
3. Browser shows permission prompt
   ↓
4. User denies permission
   ↓
5. Error displayed: "Camera permission denied. Please allow in browser settings."
   ↓
6. After 2 seconds, auto-switches to manual input
```

---

## File Modifications Summary

| File | Action | Changes |
|------|--------|---------|
| `src/mobile/utils/cameraSupport.ts` | Create | New utility for camera support detection |
| `src/mobile/components/AddDeviceModal.tsx` | Modify | Add camera check, improve error display, add hint banner |
| `src/mobile/components/QRScanner.tsx` | Modify | Add loading state, improve error messages |
| `src/mobile/i18n.ts` | Modify | Add new translation keys |

---

## Testing Plan

### Test Case 1: HTTP Connection (LAN IP)

**Setup**: Access via `http://192.168.x.x:38425/`

**Steps**:
1. Open "Add Device" modal
2. Observe hint banner in scan tab
3. Click "Start Scan"
4. Observe error message
5. Wait 2 seconds
6. Verify auto-switch to manual input tab

**Expected**: Clear error message, automatic fallback

### Test Case 2: HTTPS Connection

**Setup**: Access via HTTPS (if available)

**Steps**:
1. Open "Add Device" modal
2. Click "Start Scan"
3. Observe loading indicator
4. Verify camera viewfinder appears

**Expected**: Scanner works normally

### Test Case 3: Permission Denied

**Setup**: Any connection type

**Steps**:
1. Click "Start Scan"
2. Deny camera permission when prompted
3. Observe error message
4. Verify fallback to manual input

**Expected**: Clear permission error, fallback works

### Test Case 4: Rapid Toggle

**Setup**: Any connection type

**Steps**:
1. Click "Start Scan"
2. Immediately click "Stop Scan"
3. Repeat multiple times

**Expected**: No crashes, clean state management

---

## Benefits

### User Experience
- ✅ No more white screen confusion
- ✅ Clear, actionable error messages
- ✅ Automatic fallback to working solution
- ✅ Visual feedback during camera initialization

### Technical
- ✅ Proactive detection before attempting camera access
- ✅ Centralized camera support logic (reusable)
- ✅ Better error categorization and handling
- ✅ No backend changes required

### Compatibility
- ✅ Works on all browsers (graceful degradation)
- ✅ Supports both HTTP and HTTPS environments
- ✅ Handles permission denials properly
- ✅ Maintains existing functionality for supported browsers

---

## Backward Compatibility

- ✅ No changes to data storage
- ✅ No changes to device management logic
- ✅ Manual input still works as before
- ✅ Scanner still works when camera is available

---

## Future Enhancements (Optional)

1. **Remember user preference**: If user manually switches to "Manual Input", remember choice
2. **Copy URL from clipboard**: Add "Paste from clipboard" button in manual input
3. **Network discovery**: Auto-detect devices on same LAN (no QR needed)
4. **Custom QR scheme**: Use custom URL scheme that app can intercept

---

## Summary

**Core Fix**: Detect camera support before initialization, show clear errors, auto-fallback to manual input

**User Impact**:
- No more white screen
- Clear guidance on limitations
- Seamless fallback experience

**Technical Impact**:
- Minimal code changes
- No backend modifications
- Improved error handling throughout
