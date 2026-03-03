# Multi-Device Bug Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix QR scanner white screen and device input mixing after page refresh in mobile multi-device interface.

**Architecture:**
- QR Scanner: Replace fixed `setTimeout` with retry loop that polls for DOM element readiness
- Device Mixing: Add 2-second stabilization period after initialization to prevent race condition

**Tech Stack:** React, TypeScript, html5-qrcode, localStorage

---

## Task 1: Fix QR Scanner White Screen

**Files:**
- Modify: `src/mobile/components/QRScanner.tsx`

**Step 1: Replace useEffect with retry loop pattern**

Find the `useEffect` hook (lines 14-61) and replace it with:

```typescript
  useEffect(() => {
    isStoppedRef.current = false;

    const initWithRetry = (attempt = 0) => {
      if (isStoppedRef.current) return;

      // Check if DOM element exists
      const element = document.getElementById("qr-scanner");
      if (!element) {
        if (attempt < 20) {  // Max 20 retries = 1 second
          initTimerRef.current = window.setTimeout(
            () => initWithRetry(attempt + 1),
            50
          );
        } else {
          onError?.("Scanner element not found after retries");
        }
        return;
      }

      // DOM is ready, initialize scanner
      try {
        const scanner = new Html5Qrcode("qr-scanner");
        scannerRef.current = scanner;

        scanner
          .start(
            { facingMode: "environment" },
            { fps: 10, qrbox: { width: 250, height: 250 } },
            (decodedText) => {
              if (isStoppedRef.current) return;
              isStoppedRef.current = true;
              onScan(decodedText);
              scanner.stop().catch(() => {});
            },
            (errorMessage) => {
              const msg = String(errorMessage);
              if (msg.includes("No barcode") || msg.includes("NotFoundException")) return;
              onError?.(msg);
            },
          )
          .catch((err) => {
            const msg =
              err && typeof err === "object" && "message" in err
                ? String(err.message)
                : String(err);
            onError?.(`Camera init failed: ${msg}`);
          });
      } catch (err) {
        onError?.(
          `Scanner init failed: ${err instanceof Error ? err.message : String(err)}`
        );
      }
    };

    initWithRetry();

    return () => {
      isStoppedRef.current = true;
      if (initTimerRef.current !== null) {
        window.clearTimeout(initTimerRef.current);
        initTimerRef.current = null;
      }
      const scanner = scannerRef.current;
      scannerRef.current = null;
      scanner?.stop().catch(() => {});
    };
  }, [onError, onScan]);
```

**Step 2: Build to verify TypeScript compilation**

Run: `pnpm build`
Expected: Build succeeds with no errors

**Step 3: Commit**

```bash
git add src/mobile/components/QRScanner.tsx
git commit -m "fix(mobile): replace setTimeout with retry loop for QR scanner DOM readiness

- Polls for #qr-scanner element existence every 50ms
- Max 20 retries (1 second) before showing error
- Ensures DOM is ready before Html5Qrcode initialization

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Add Stabilization State to App

**Files:**
- Modify: `src/mobile/App.tsx`

**Step 1: Add isStabilizing state**

Find the state declarations at the top of the component (after line 28). Add new state after `showRename`:

```typescript
  const [showRename, setShowRename] = useState(false);
  const [isStabilizing, setIsStabilizing] = useState(true);
```

**Step 2: Add stabilization useEffect**

Find the existing `useEffect` hooks (after line 70). Add this new effect after the i18n initialization effect:

```typescript
  useEffect(() => {
    if (!isInitialized) return;

    const timer = window.setTimeout(() => {
      setIsStabilizing(false);
    }, 2000);  // 2 second stabilization period

    return () => window.clearTimeout(timer);
  }, [isInitialized]);
```

**Step 3: Build to verify TypeScript compilation**

Run: `pnpm build`
Expected: Build succeeds with no errors

**Step 4: Commit**

```bash
git add src/mobile/App.tsx
git commit -m "feat(mobile): add initialization stabilization period

- Adds 2-second stabilization period after device manager init
- Prevents sending messages during race condition window
- Will be integrated with send button and input in next step

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 3: Integrate Stabilization with Send Button

**Files:**
- Modify: `src/mobile/App.tsx`

**Step 1: Update canSend calculation**

Find the `canSend` useMemo (around line 95). Modify to include `isStabilizing`:

```typescript
  const canSend = useMemo(() => {
    if (isStabilizing) return false;
    const active = devices.find((d) => d.id === activeDeviceId);
    return (
      !!active &&
      active.status === "connected" &&
      !isSending &&
      text.trim().length > 0 &&
      text.length <= 10000
    );
  }, [activeDeviceId, devices, isSending, text, isStabilizing]);
```

**Step 2: Update SendButton isSending prop**

Find the `SendButton` component usage (around line 255). Update the `isSending` prop:

```typescript
            <SendButton
              onClick={handleSend}
              disabled={!canSend}
              isSending={isSending || isStabilizing}
            />
```

**Step 3: Build to verify TypeScript compilation**

Run: `pnpm build`
Expected: Build succeeds with no errors

**Step 4: Commit**

```bash
git add src/mobile/App.tsx
git commit -m "feat(mobile): integrate stabilization with send button

- Send button disabled during 2-second stabilization period
- Shows loading state while reconnecting after refresh
- Prevents sending to wrong device during race condition

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 4: Disable Input During Stabilization

**Files:**
- Modify: `src/mobile/App.tsx`

**Step 1: Update TextInput disabled prop**

Find the `TextInput` component usage (around line 243). Update the `disabled` prop:

```typescript
          <TextInput
            value={text}
            onChange={setText}
            onKeyDown={handleKeyDown}
            disabled={
              !devices.find((d) => d.id === activeDeviceId)?.status ||
              devices.find((d) => d.id === activeDeviceId)?.status !== "connected" ||
              isStabilizing
            }
            onOpenSettings={handleOpenSettings}
          />
```

Or use a cleaner extracted variable. Replace the entire TextInput section with:

```typescript
          <TextInput
            value={text}
            onChange={setText}
            onKeyDown={handleKeyDown}
            disabled={
              isStabilizing ||
              devices.find((d) => d.id === activeDeviceId)?.status !== "connected"
            }
            onOpenSettings={handleOpenSettings}
          />
```

**Step 2: Build to verify TypeScript compilation**

Run: `pnpm build`
Expected: Build succeeds with no errors

**Step 3: Commit**

```bash
git add src/mobile/App.tsx
git commit -m "feat(mobile): disable input during stabilization period

- Text input disabled during 2-second stabilization
- Provides clear feedback that reconnection is in progress
- Completes the device mixing fix implementation

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 5: Manual Testing

**Step 1: Test QR Scanner on Real Device**

1. Run `pnpm tauri dev`
2. Scan QR code with mobile device
3. Click "Start Scan" button
4. Expected: Camera viewfinder appears within 1 second, permission requested
5. Try rapid stop/start toggle
6. Expected: Each initialization succeeds

**Step 2: Test Device Mixing Fix**

1. Add 2 devices (e.g., PC-A and PC-B)
2. Refresh the page
3. Observe send button showing loading state for 2 seconds
4. Try to send during stabilization
5. Expected: Button disabled, cannot send
6. Wait 2 seconds, then send message
7. Expected: Message sent to the active device shown in UI

**Step 3: Edge Cases**

1. Clear localStorage, refresh
2. Expected: Auto-detects current PC device
3. Single device scenario
4. Expected: No stabilization issues

---

## Task 6: Optional - Add i18n for Reconnecting State

**Files:**
- Modify: `src/mobile/i18n.ts`

**Step 1: Add translation key**

Find the `zh` translations object and add to `notifications` section:

```typescript
    notifications: {
      lastMessageRestored: "上次发送的内容已恢复",
      reconnecting: "正在重新连接...",
    },
```

Find the `en` translations object and add:

```typescript
    notifications: {
      lastMessageRestored: "Last message restored",
      reconnecting: "Reconnecting...",
    },
```

**Step 2: Commit**

```bash
git add src/mobile/i18n.ts
git commit -m "feat(mobile): add i18n for reconnection state

- Add 'reconnecting' translation for zh and en
- Can be used in future for toast message during stabilization

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Summary

**Total Changes:**
- 1 file modified for QR scanner fix
- 1 file modified for device mixing fix
- 1 file optionally modified for i18n

**Testing:** Manual testing required on real mobile device

**Backward Compatibility:** Fully compatible, no data migration needed
