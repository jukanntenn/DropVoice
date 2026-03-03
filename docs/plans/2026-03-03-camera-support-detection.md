# Camera Support Detection and Fallback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix QR scanner white screen by detecting camera support and gracefully falling back to manual input when camera is unavailable.

**Architecture:** Add camera support detection utility, integrate check before scanner initialization, show user-friendly errors, and auto-switch to manual input on failure.

**Tech Stack:** React, TypeScript, html5-qrcode, navigator.mediaDevices API

---

## Task 1: Create Camera Support Detection Utility

**Files:**
- Create: `src/mobile/utils/cameraSupport.ts`

**Step 1: Create the utility file with TypeScript interface**

Create file `src/mobile/utils/cameraSupport.ts`:

```typescript
export interface CameraSupportResult {
  supported: boolean;
  reason?: string;
  userMessage?: string;
}

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
  const isLocalhost =
    window.location.hostname === "localhost" ||
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
      video: { facingMode: "environment" },
    });

    // Immediately stop the stream (we just wanted to test access)
    stream.getTracks().forEach((track) => track.stop());

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

**Step 2: Build to verify TypeScript compilation**

Run: `pnpm build`

Expected: Build succeeds with no errors

**Step 3: Commit**

```bash
git add src/mobile/utils/cameraSupport.ts
git commit -m "feat(mobile): add camera support detection utility

- Check MediaDevices API availability
- Verify secure context (HTTPS or localhost)
- Test actual camera access
- Return user-friendly error messages

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Update AddDeviceModal - Import and Handler

**Files:**
- Modify: `src/mobile/components/AddDeviceModal.tsx`

**Step 1: Add import for camera support utility**

Find line 1-4 (imports section), add after line 3:

```typescript
import { checkCameraSupport } from "../utils/cameraSupport";
```

**Step 2: Replace start scan button handler**

Find the scan button onClick handler (around line 147):

```typescript
onClick={() => setIsScanning((v) => !v)}
```

Replace with:

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

**Step 3: Build to verify TypeScript compilation**

Run: `pnpm build`

Expected: Build succeeds with no errors

**Step 4: Commit**

```bash
git add src/mobile/components/AddDeviceModal.tsx
git commit -m "feat(mobile): integrate camera support check in AddDeviceModal

- Check camera support before starting scanner
- Show error message when camera unavailable
- Auto-switch to manual input after 2 seconds
- Prevent white screen on HTTP connections

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 3: Update AddDeviceModal - Add Hint Banner

**Files:**
- Modify: `src/mobile/components/AddDeviceModal.tsx`

**Step 1: Add hint banner in scan tab**

Find the scan tab content (around line 128-143), the section starting with:

```typescript
{tab === "scan" ? (
  <div className="space-y-4">
    {isScanning ? (
```

Add this banner immediately after `<div className="space-y-4">` and before `{isScanning ? (`:

```typescript
{tab === "scan" ? (
  <div className="space-y-4">
    <div className="mb-4 rounded-2xl bg-amber-50 px-4 py-3 text-sm text-amber-800 dark:bg-amber-900/30 dark:text-amber-200">
      <div className="flex items-start gap-2">
        <svg
          className="h-5 w-5 flex-shrink-0 mt-0.5"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fillRule="evenodd"
            d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
            clipRule="evenodd"
          />
        </svg>
        <div>
          <p className="font-medium">提示</p>
          <p className="mt-1">
            摄像头需要 HTTPS 或 localhost 才能使用。如无法扫描，请使用手动输入。
          </p>
        </div>
      </div>
    </div>
    {isScanning ? (
```

**Step 2: Build to verify TypeScript compilation**

Run: `pnpm build`

Expected: Build succeeds with no errors

**Step 3: Commit**

```bash
git add src/mobile/components/AddDeviceModal.tsx
git commit -m "feat(mobile): add HTTPS requirement hint banner in scan tab

- Show warning about camera HTTPS requirement
- Guide users to manual input alternative
- Improve user awareness before attempting scan

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 4: Update AddDeviceModal - Improve Error Display

**Files:**
- Modify: `src/mobile/components/AddDeviceModal.tsx`

**Step 1: Enhance error display with icon and structure**

Find the error display section (around line 199-203):

```typescript
{error ? (
  <div className="mt-4 rounded-2xl bg-red-100 px-4 py-3 text-sm text-red-700 dark:bg-red-900/30 dark:text-red-200">
    {error}
  </div>
) : null}
```

Replace with:

```typescript
{error && (
  <div className="mt-4 rounded-2xl bg-red-100 px-4 py-3 text-sm text-red-700 dark:bg-red-900/30 dark:text-red-200">
    <div className="flex items-start gap-2">
      <svg
        className="h-5 w-5 flex-shrink-0 mt-0.5"
        fill="currentColor"
        viewBox="0 0 20 20"
      >
        <path
          fillRule="evenodd"
          d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
          clipRule="evenodd"
        />
      </svg>
      <div>
        <p className="font-medium">无法访问摄像头</p>
        <p className="mt-1">{error}</p>
      </div>
    </div>
  </div>
)}
```

**Step 2: Build to verify TypeScript compilation**

Run: `pnpm build`

Expected: Build succeeds with no errors

**Step 3: Commit**

```bash
git add src/mobile/components/AddDeviceModal.tsx
git commit -m "feat(mobile): improve error display with icon and structure

- Add error icon for visual clarity
- Separate error title from message
- Better visual hierarchy

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 5: Update QRScanner - Add Loading State

**Files:**
- Modify: `src/mobile/components/QRScanner.tsx`

**Step 1: Add loading state variable**

Find line 11 (after `const initTimerRef = useRef<number | null>(null);`), add:

```typescript
const [isLoading, setIsLoading] = useState(true);
```

**Step 2: Update scanner initialization to manage loading state**

Find the scanner.start() call (around line 33-53), update the success callback and catch block:

Replace the entire scanner.start() chain (lines 33-53) with:

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

**Step 3: Update render to show loading indicator**

Find the return statement (around line 73-78):

```typescript
  return (
    <div
      id="qr-scanner"
      className="w-full aspect-square bg-black rounded-2xl overflow-hidden"
    />
  );
```

Replace with:

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

**Step 4: Build to verify TypeScript compilation**

Run: `pnpm build`

Expected: Build succeeds with no errors

**Step 5: Commit**

```bash
git add src/mobile/components/QRScanner.tsx
git commit -m "feat(mobile): add loading state to QRScanner component

- Show loading indicator during camera initialization
- Improve error messages with specific causes
- Better user feedback during scanner startup

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 6: Update i18n - Add Translation Keys

**Files:**
- Modify: `src/mobile/i18n.ts`

**Step 1: Add Chinese translations**

Find the Chinese translations object (around line 10-50), locate the `devices` section. Add these keys to the `devices` object:

```typescript
    cameraRequiresHttps: "摄像头需要 HTTPS 或 localhost 才能使用",
    switchedToManualInput: "已自动切换到手动输入",
    cameraPermissionDenied: "摄像头权限被拒绝。请在浏览器设置中允许访问摄像头。",
    cameraNotSupported: "您的浏览器不支持摄像头功能",
    cameraNotFound: "未找到摄像头设备。",
    cameraInUse: "摄像头被其他应用程序占用。",
    startingCamera: "正在启动摄像头...",
    cameraInitFailed: "摄像头初始化失败",
    scanHintHttps: "摄像头需要 HTTPS 或 localhost 才能使用。如无法扫描，请使用手动输入。",
```

**Step 2: Add English translations**

Find the English translations object (around line 60-100), locate the `devices` section. Add these keys to the `devices` object:

```typescript
    cameraRequiresHttps: "Camera requires HTTPS or localhost",
    switchedToManualInput: "Switched to manual input",
    cameraPermissionDenied: "Camera permission denied. Please allow camera access in browser settings.",
    cameraNotSupported: "Your browser doesn't support camera",
    cameraNotFound: "No camera device found.",
    cameraInUse: "Camera is being used by another application.",
    startingCamera: "Starting camera...",
    cameraInitFailed: "Camera initialization failed",
    scanHintHttps: "Camera requires HTTPS or localhost. Please use manual input if scanning is unavailable.",
```

**Step 3: Build to verify TypeScript compilation**

Run: `pnpm build`

Expected: Build succeeds with no errors

**Step 4: Commit**

```bash
git add src/mobile/i18n.ts
git commit -m "feat(mobile): add camera-related i18n translation keys

- Add translations for camera errors and hints
- Support both Chinese and English
- Prepare for future i18n usage

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 7: Manual Testing - HTTP Connection

**Step 1: Start development server**

Run: `pnpm tauri dev`

Wait for the app to start and display QR code.

**Step 2: Access mobile interface via HTTP**

1. Use your mobile device on the same LAN
2. Scan the QR code or manually enter the URL (e.g., `http://192.168.x.x:38425/`)
3. Open the mobile interface

**Step 3: Test camera detection**

1. Click "添加设备" (Add Device) button
2. Observe the hint banner in the scan tab
3. Click "开始扫描" (Start Scan) button
4. Expected: Error message appears: "摄像头需要 HTTPS 或 localhost 才能使用。已自动切换到手动输入。"
5. Wait 2 seconds
6. Expected: Automatically switches to "手动输入" (Manual Input) tab

**Step 4: Verify manual input works**

1. Enter a valid WebSocket URL in the input field
2. Click "添加" (Add) button
3. Expected: Device is added successfully

**Step 5: Test stop scan button**

1. Click "添加设备" again
2. Click "开始扫描"
3. Immediately click "停止扫描" (Stop Scan) button
4. Expected: Button toggles correctly, no errors

---

## Task 8: Manual Testing - HTTPS/Localhost (Optional)

**Note**: This test requires HTTPS setup or localhost access.

**Step 1: Test on localhost (if applicable)**

1. Access via `http://localhost:5173/` or `http://127.0.0.1:5173/`
2. Click "添加设备"
3. Click "开始扫描"
4. Expected: Camera permission prompt appears (if browser allows)
5. Allow permission
6. Expected: Camera viewfinder appears with loading indicator

**Step 2: Test permission denial**

1. Click "开始扫描"
2. Deny camera permission when prompted
3. Expected: Error message "摄像头权限被拒绝。请在浏览器设置中允许访问摄像头。"
4. Expected: Auto-switch to manual input after 2 seconds

---

## Task 9: Final Build Verification

**Step 1: Run production build**

Run: `pnpm build`

Expected: Build succeeds with no errors

**Step 2: Run Tauri build (optional)**

Run: `pnpm tauri build`

Expected: Build completes successfully

---

## Summary

**Total Changes:**
- 1 new file created (cameraSupport.ts)
- 3 files modified (AddDeviceModal.tsx, QRScanner.tsx, i18n.ts)
- 6 commits total

**Testing:** Manual testing required on real mobile device with HTTP connection

**Backward Compatibility:** Fully compatible, no data migration needed

**User Impact:**
- ✅ No more white screen on HTTP connections
- ✅ Clear error messages with guidance
- ✅ Automatic fallback to manual input
- ✅ Loading indicator during camera initialization
