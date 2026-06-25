# Mobile Multi-Device Dot Switcher Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the mobile app's top device-tab strip with an IP-colored dot indicator row beneath the textarea, move Add/Remove into a kebab overflow menu, surface the active device's IP in `document.title`, and remove custom device renaming entirely.

**Architecture:** Pure color logic (`deviceColor.ts`) is TDD'd with vitest. Three new presentational components (`DeviceDots`, `ConnectionPill`, `DeviceMenu`) are composed into `App.tsx`, replacing `DeviceTabs`/`DeviceCard`/`DeviceNameEditor` (deleted). Device order becomes stable insertion order (recency sort removed). Status is encoded on inactive dots via opacity/pulse while preserving each device's IP hue; the active device's error state shows in a pill above the textarea.

**Tech Stack:** React 18, TypeScript (strict, `noUnusedLocals`/`noUnusedParameters`), Tailwind CSS v4, Vite 6, vitest (added here), lucide-react, react-i18next. Mobile entry: `src/mobile/index.html`.

**Testing strategy:** The repo has **no** existing test runner. This plan adds **vitest** and TDDs the pure `deviceColor.ts` (high-value edge cases: URL parsing, golden-angle math, hostname fallback). React components are presentational and are verified by `tsc` type-check + manual visual check (the repo has no DOM/component test tooling, and adding it is out of scope). Every task ends with a type-check or build + commit.

**Key conventions read from the codebase:**
- `Device` type (`src/mobile/types/device.ts`): `{ id, name, url, status: "connected"|"disconnected"|"connecting", lastConnected, hasExhaustedRetries?, errorType?: "unreachable"|"refused"|"timeout"|"unknown" }`. IP is derived from `device.url` (e.g. `ws://192.168.1.100:38425/ws`).
- `tsconfig.json` already excludes `src/**/*.test.ts(x)` from the build and has `noUnusedLocals`/`noUnusedParameters` ON — so test files won't break `pnpm build`, and unused imports/vars WILL fail the build. Clean up imports religiously.
- `useSettings()` (`src/mobile/hooks/useSettings.ts`) returns `{ resolvedTheme: "light"|"dark", ... }`, already reactive to system theme + `matchMedia`. It also applies the `dark` class to `<html>` — `App.tsx` MUST keep calling it.
- Locales are nested JSON under `devices.*` in `src/mobile/locales/{en,zh,zh-TW,ja}.json`.

---

## Task 1: Feature branch + vitest setup

**Files:**
- Modify: `package.json`
- Create: `vitest.config.ts`

**Step 1: Create and switch to a feature branch**

Run: `git checkout -b feat/mobile-device-dots`
Expected: `Switched to a new branch 'feat/mobile-device-dots'`

**Step 2: Install vitest as a dev dependency**

Run: `pnpm add -D vitest`
Expected: vitest added under `devDependencies`; lockfile updated.

**Step 3: Create `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
```

**Step 4: Add test scripts to `package.json`**

In the `"scripts"` block, add:
```json
    "test": "vitest run",
    "test:watch": "vitest",
```
(after the `"preview"` line is fine).

**Step 5: Verify the runner works (0 tests)**

Run: `pnpm test`
Expected: completes with no test files found (exit 0). If it errors on config, fix before continuing.

**Step 6: Commit**

```bash
git add package.json pnpm-lock.yaml vitest.config.ts
git commit -m "chore: add vitest for unit testing"
```

---

## Task 2: `deviceColor.ts` pure utilities (TDD)

Pure functions with no DOM/React. This is the only real logic in the feature, so it gets unit tests.

**Files:**
- Create: `src/mobile/utils/deviceColor.ts`
- Test: `src/mobile/utils/deviceColor.test.ts`

**Step 1: Write the failing tests**

Create `src/mobile/utils/deviceColor.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { Device } from "../types";
import { getDeviceColor, getDeviceHostLabel, getDeviceHue } from "./deviceColor";

function dev(url: string): Device {
  return { id: "x", name: "n", url, status: "connected", lastConnected: 0 };
}

describe("getDeviceHue", () => {
  it("derives hue from the last IPv4 octet via the golden angle", () => {
    expect(getDeviceHue(dev("ws://192.168.1.0:38425/ws"))).toBe(0);
    expect(getDeviceHue(dev("ws://192.168.1.100:38425/ws"))).toBe(71);
    expect(getDeviceHue(dev("ws://192.168.1.101:38425/ws"))).toBe(208);
    expect(getDeviceHue(dev("ws://192.168.1.102:38425/ws"))).toBe(346);
  });

  it("spreads consecutive octets widely (>=90 degrees apart)", () => {
    const h = (octet: number) =>
      getDeviceHue(dev(`ws://10.0.0.${octet}:1/ws`));
    for (let i = 0; i < 20; i++) {
      const d = Math.abs(h(i) - h(i + 1));
      const spread = Math.min(d, 360 - d);
      expect(spread).toBeGreaterThanOrEqual(90);
    }
  });

  it("is deterministic for the same device", () => {
    const a = getDeviceHue(dev("ws://192.168.0.42:1/ws"));
    const b = getDeviceHue(dev("ws://192.168.0.42:1/ws"));
    expect(a).toBe(b);
  });

  it("falls back to a hostname hash for non-IP hosts (stable, in range)", () => {
    const hue = getDeviceHue(dev("ws://my-pc.local:9000/ws"));
    expect(Number.isInteger(hue)).toBe(true);
    expect(hue).toBeGreaterThanOrEqual(0);
    expect(hue).toBeLessThan(360);
    expect(getDeviceHue(dev("ws://my-pc.local:9000/ws"))).toBe(hue);
  });

  it("falls back to hashing the raw url when the URL is malformed", () => {
    const hue = getDeviceHue(dev("not-a-url"));
    expect(hue).toBeGreaterThanOrEqual(0);
    expect(hue).toBeLessThan(360);
  });

  it("different hostnames usually map to different hues", () => {
    const a = getDeviceHue(dev("ws://alpha.local:1/ws"));
    const b = getDeviceHue(dev("ws://beta.local:1/ws"));
    expect(a).not.toBe(b);
  });
});

describe("getDeviceColor", () => {
  it("returns an hsl string with theme-aware saturation/lightness", () => {
    expect(getDeviceColor(dev("ws://192.168.1.100:1/ws"), false)).toBe("hsl(71 60% 52%)");
    expect(getDeviceColor(dev("ws://192.168.1.100:1/ws"), true)).toBe("hsl(71 70% 62%)");
  });
});

describe("getDeviceHostLabel", () => {
  it("returns the IPv4 hostname without port", () => {
    expect(getDeviceHostLabel(dev("ws://192.168.1.100:38425/ws"))).toBe("192.168.1.100");
  });

  it("returns a non-IP hostname as-is", () => {
    expect(getDeviceHostLabel(dev("ws://my-pc.local:9000/ws"))).toBe("my-pc.local");
  });

  it("falls back to the raw url when parsing fails", () => {
    expect(getDeviceHostLabel(dev("garbage"))).toBe("garbage");
  });
});
```

**Step 2: Run tests to verify they fail**

Run: `pnpm test`
Expected: FAIL — `Failed to resolve import "./deviceColor"` (module not found).

**Step 3: Implement `src/mobile/utils/deviceColor.ts`**

```ts
import type { Device } from "../types";

const GOLDEN_ANGLE = 137.508;

function hostnameOf(url: string): string | null {
  try {
    return new URL(url).hostname;
  } catch {
    return null;
  }
}

function isIpv4(host: string): boolean {
  return /^\d+\.\d+\.\d+\.\d+$/.test(host);
}

function lastOctet(host: string): number {
  const part = host.split(".")[3];
  const n = Number(part);
  return Number.isFinite(n) ? n : 0;
}

function hashString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (h * 31 + s.charCodeAt(i)) >>> 0;
  }
  return h;
}

/** Hue [0,360) for a device, derived from its last IPv4 octet (golden angle) or hostname hash. */
export function getDeviceHue(device: Device): number {
  const host = hostnameOf(device.url);
  if (host && isIpv4(host)) {
    return Math.round((lastOctet(host) * GOLDEN_ANGLE) % 360);
  }
  return hashString(host ?? device.url) % 360;
}

/** CSS hsl() color for a device, with saturation/lightness chosen for the theme. */
export function getDeviceColor(device: Device, dark: boolean): string {
  const hue = getDeviceHue(device);
  return dark ? `hsl(${hue} 70% 62%)` : `hsl(${hue} 60% 52%)`;
}

/** Hostname for display (document.title, aria-label, confirm dialog). IPv4 literal if IP, else hostname. */
export function getDeviceHostLabel(device: Device): string {
  return hostnameOf(device.url) ?? device.url;
}
```

**Step 4: Run tests to verify they pass**

Run: `pnpm test`
Expected: PASS — all `deviceColor` tests green.

**Step 5: Type-check the production source (test files are excluded by tsconfig)**

Run: `pnpm exec tsc --noEmit`
Expected: no errors.

**Step 6: Commit**

```bash
git add src/mobile/utils/deviceColor.ts src/mobile/utils/deviceColor.test.ts
git commit -m "feat(mobile): add IP-derived device color utilities"
```

---

## Task 3: `DeviceDots.tsx` component

Presentational; verified by type-check now, visually in Task 8.

**Files:**
- Create: `src/mobile/components/DeviceDots.tsx`

**Step 1: Create the component**

Create `src/mobile/components/DeviceDots.tsx`:

```tsx
import type { Device } from "../types";
import { getDeviceColor, getDeviceHostLabel } from "../utils/deviceColor";

interface DeviceDotsProps {
  devices: Device[];
  activeDeviceId: string | null;
  dark: boolean;
  onSelect: (deviceId: string) => void;
}

/**
 * Dot indicator row for switching devices. Hidden when <=1 device.
 * Active dot = solid + larger; inactive = hollow + smaller, in each device's IP color.
 * Inactive status is encoded as: connecting -> pulse, disconnected -> dimmed. Hue is preserved.
 */
export function DeviceDots({ devices, activeDeviceId, dark, onSelect }: DeviceDotsProps) {
  if (devices.length <= 1) return null;

  return (
    <div className="mt-3 flex items-center justify-center gap-3">
      {devices.map((device) => {
        const active = device.id === activeDeviceId;
        const color = getDeviceColor(device, dark);
        const connecting = device.status === "connecting";
        const dimmed = !active && device.status === "disconnected";
        const sizeCls = active ? "h-3.5 w-3.5" : "h-2.5 w-2.5";
        const dotStyle = active
          ? { backgroundColor: color }
          : { borderColor: color, backgroundColor: "transparent" };
        const stateCls = [
          connecting ? "animate-pulse" : "",
          dimmed ? "opacity-40" : "",
        ].join(" ");
        return (
          <button
            key={device.id}
            type="button"
            onClick={() => onSelect(device.id)}
            aria-label={getDeviceHostLabel(device)}
            aria-current={active ? "true" : undefined}
            className="flex h-9 w-9 items-center justify-center rounded-full"
          >
            <span
              style={dotStyle}
              className={[
                "rounded-full",
                sizeCls,
                active ? "" : "border-2",
                stateCls,
              ].join(" ")}
            />
          </button>
        );
      })}
    </div>
  );
}
```

**Step 2: Type-check**

Run: `pnpm exec tsc --noEmit`
Expected: no errors (component is unused but exported, so `noUnusedLocals` is satisfied).

**Step 3: Commit**

```bash
git add src/mobile/components/DeviceDots.tsx
git commit -m "feat(mobile): add DeviceDots indicator component"
```

---

## Task 4: `ConnectionPill.tsx` + `DeviceMenu.tsx` components

**Files:**
- Create: `src/mobile/components/ConnectionPill.tsx`
- Create: `src/mobile/components/DeviceMenu.tsx`

**Step 1: Create `ConnectionPill.tsx`**

```tsx
import { useTranslation } from "react-i18next";
import type { Device } from "../types";

interface ConnectionPillProps {
  device: Device | null;
  onRetry: () => void;
}

/** Shows the active device's error label + Retry. Renders nothing unless the device is disconnected. */
export function ConnectionPill({ device, onRetry }: ConnectionPillProps) {
  const { t } = useTranslation();
  if (!device || device.status !== "disconnected") return null;

  const errorLabel = device.errorType
    ? t(
        `devices.error${device.errorType.charAt(0).toUpperCase()}${device.errorType.slice(1)}`,
      )
    : t("devices.errorUnknown");

  return (
    <div className="flex items-center gap-2 rounded-full bg-red-50 px-3 py-1.5 text-xs font-medium text-red-600 dark:bg-red-900/30 dark:text-red-300">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 20 20"
        fill="currentColor"
        className="h-3.5 w-3.5 flex-shrink-0"
      >
        <path
          fillRule="evenodd"
          d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
          clipRule="evenodd"
        />
      </svg>
      <span>{errorLabel}</span>
      <button
        type="button"
        onClick={onRetry}
        className="font-semibold text-red-700 hover:underline dark:text-red-200"
      >
        {t("devices.retry")}
      </button>
    </div>
  );
}
```

**Step 2: Create `DeviceMenu.tsx` (kebab overflow: Add / Remove)**

```tsx
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { MoreVertical, Plus, Trash2 } from "lucide-react";

interface DeviceMenuProps {
  canRemove: boolean;
  onAdd: () => void;
  onRemove: () => void;
}

/** Kebab overflow menu with Add device and Remove (active) device actions. */
export function DeviceMenu({ canRemove, onAdd, onRemove }: DeviceMenuProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const choose = (fn: () => void) => {
    fn();
    setOpen(false);
  };

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-label={t("devices.menu")}
        aria-haspopup="menu"
        aria-expanded={open}
        className="flex h-9 w-9 items-center justify-center rounded-full text-slate-500 transition-colors hover:bg-black/5 hover:text-slate-700 dark:text-slate-400 dark:hover:bg-white/5 dark:hover:text-slate-200"
      >
        <MoreVertical className="h-5 w-5" />
      </button>
      {open && (
        <div
          role="menu"
          className="absolute right-0 top-full z-20 mt-1 w-44 overflow-hidden rounded-2xl border border-slate-200 bg-white py-1 shadow-lg dark:border-slate-700 dark:bg-slate-800"
        >
          <button
            type="button"
            role="menuitem"
            onClick={() => choose(onAdd)}
            className="flex w-full items-center gap-2 px-4 py-2.5 text-sm text-slate-700 hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-700"
          >
            <Plus className="h-4 w-4" /> {t("devices.addDevice")}
          </button>
          <button
            type="button"
            role="menuitem"
            disabled={!canRemove}
            onClick={() => canRemove && choose(onRemove)}
            className="flex w-full items-center gap-2 px-4 py-2.5 text-sm text-red-600 hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-40 dark:text-red-400 dark:hover:bg-red-900/30"
          >
            <Trash2 className="h-4 w-4" /> {t("devices.removeDevice")}
          </button>
        </div>
      )}
    </div>
  );
}
```

**Step 3: Type-check**

Run: `pnpm exec tsc --noEmit`
Expected: no errors. (Note: `devices.menu` and `devices.removeDevice` keys are added in Task 7; i18next returns the key string if missing, so the build is unaffected. The keys will exist before final verification.)

**Step 4: Commit**

```bash
git add src/mobile/components/ConnectionPill.tsx src/mobile/components/DeviceMenu.tsx
git commit -m "feat(mobile): add ConnectionPill and kebab DeviceMenu components"
```

---

## Task 5: Stable insertion order in `useDeviceManager`

Remove the recency sort so device positions (and thus dots) are stable.

**Files:**
- Modify: `src/mobile/hooks/useDeviceManager.ts` (around line 49)

**Step 1: Remove the recency sort**

In `src/mobile/hooks/useDeviceManager.ts`, delete this line (inside the init `useEffect`, after the auto-device block):

```ts
    loadedDevices.sort((a, b) => b.lastConnected - a.lastConnected);
```

The stored array order (append-on-add in `useMultiWebSocket.addDevice`) becomes the dot order. No other change needed — `setDevicesWithActiveGuard` and the persist effect preserve order.

**Step 2: Type-check + build**

Run: `pnpm exec tsc --noEmit`
Expected: no errors.

**Step 3: Commit**

```bash
git add src/mobile/hooks/useDeviceManager.ts
git commit -m "refactor(mobile): use stable insertion order for devices"
```

---

## Task 6: Integrate the new UI in `App.tsx` + trim hook/modal + delete old components

This is the wiring task. It removes renaming, the swipe gesture, and the old tab components, and composes the new top row + dots. Several files change together so the app stays compilable.

**Files:**
- Modify: `src/mobile/App.tsx`
- Modify: `src/mobile/hooks/useMultiWebSocket.ts`
- Modify: `src/mobile/components/AddDeviceModal.tsx`
- Delete: `src/mobile/components/DeviceTabs.tsx`
- Delete: `src/mobile/components/DeviceCard.tsx`
- Delete: `src/mobile/components/DeviceNameEditor.tsx`

### Step 1: `useMultiWebSocket.ts` — remove rename, simplify addDevice

In `src/mobile/hooks/useMultiWebSocket.ts`:

1. In the `UseMultiWebSocketReturn` interface, delete the line:
   ```ts
     renameDevice: (deviceId: string, newName: string) => void;
   ```

2. Change the `addDevice` signature and name handling. Replace:
   ```ts
     const addDevice = useCallback(
       (url: string, name?: string): AddDeviceResult => {
   ```
   with:
   ```ts
     const addDevice = useCallback(
       (url: string): AddDeviceResult => {
   ```
   and in the `device` object inside it, replace:
   ```ts
         name: name?.trim() ? name.trim() : getDefaultDeviceName(url),
   ```
   with:
   ```ts
         name: getDefaultDeviceName(url),
   ```

3. Delete the entire `renameDevice` `useCallback` block:
   ```ts
     const renameDevice = useCallback(
       (deviceId: string, newName: string) => {
         const trimmed = newName.trim();
         if (!trimmed) return;
         setDevices(
           devicesRef.current.map((d) => (d.id === deviceId ? { ...d, name: trimmed } : d)),
         );
       },
       [setDevices],
     );
   ```

4. In the returned object of the final `useMemo`, delete:
   ```ts
       renameDevice,
   ```
   and in its dependency array, delete:
   ```ts
       renameDevice,
   ```

### Step 2: `AddDeviceModal.tsx` — remove the name field

In `src/mobile/components/AddDeviceModal.tsx`:

1. Change the prop type:
   ```ts
     onAdd: (url: string, name?: string) => AddDeviceResult;
   ```
   →
   ```ts
     onAdd: (url: string) => AddDeviceResult;
   ```

2. Delete the `nameInput` state line:
   ```ts
   const [nameInput, setNameInput] = useState("");
   ```

3. In `reset`, delete `setNameInput("");`.

4. In `submitUrl`, change:
   ```ts
       const result = onAdd(wsUrl, nameInput.trim() ? nameInput.trim() : undefined);
   ```
   →
   ```ts
       const result = onAdd(wsUrl);
   ```
   and remove `nameInput` from that `useCallback`'s dep array (it becomes `[close, normalizedUrl, onAdd, t]`).

5. Delete the entire name-field JSX block:
   ```tsx
               <div>
                 <label className="mb-2 block text-sm font-medium text-slate-700 dark:text-slate-200">
                   {t("devices.deviceNameOptional")}
                 </label>
                 <input
                   type="text"
                   value={nameInput}
                   onChange={(e) => setNameInput(e.target.value)}
                   placeholder={t("devices.namePlaceholder")}
                   className="w-full rounded-2xl border border-slate-300 bg-white px-4 py-3 text-slate-900 outline-none focus:ring-2 focus:ring-primary/50 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100"
                 />
               </div>
   ```

### Step 3: `App.tsx` — imports, state, handlers, effects, and JSX

Apply these edits to `src/mobile/App.tsx`:

1. **Imports.** Remove:
   ```ts
   import { DeviceNameEditor } from "./components/DeviceNameEditor";
   import { DeviceTabs } from "./components/DeviceTabs";
   ```
   Add (place with the other component imports near the top):
   ```ts
   import { ConnectionPill } from "./components/ConnectionPill";
   import { DeviceDots } from "./components/DeviceDots";
   import { DeviceMenu } from "./components/DeviceMenu";
   import { getDeviceHostLabel } from "./utils/deviceColor";
   ```

2. **Settings hook.** Change:
   ```ts
     useSettings();
   ```
   →
   ```ts
     const { resolvedTheme } = useSettings();
   ```
   (Keeps the theme side-effect running and gives us the resolved value.)

3. **State.** Delete these two lines:
   ```ts
     const [renameDeviceId, setRenameDeviceId] = useState<string | null>(null);
     const [showRename, setShowRename] = useState(false);
   ```
   Keep `showAddDevice` and `removeDeviceId`.

4. **Refs.** Delete the swipe ref:
   ```ts
     const touchStartRef = useRef<{ x: number; y: number } | null>(null);
   ```

5. **Destructure.** In the `useMultiWebSocket(...)` destructure, remove `renameDevice` and `retryDevice` is still needed. The block currently includes:
   ```ts
     renameDevice,
     setActiveDevice,
     retryDevice,
     sendToActive,
   ```
   → remove the `renameDevice,` line (keep the rest).

6. **Swipe handlers.** Delete the entire `handleTouchStart` and `handleTouchEnd` `useCallback` blocks (the two big callbacks dealing with `touchStartRef` / `dx` / `dy` / `sorted`).

7. **Device-switch draft effect.** In the `useEffect` that handles draft save/restore on switch, find the toast line:
   ```ts
         showToast(t("devices.switchedTo", { name: switchedDevice.name }), "success", 1500);
   ```
   →
   ```ts
         showToast(
           t("devices.switchedTo", { ip: getDeviceHostLabel(switchedDevice) }),
           "success",
           1500,
         );
   ```

8. **Add a `document.title` effect.** Add this new `useEffect` (anywhere among the other effects, e.g. right after the switch-draft effect):
   ```ts
     // Reflect the active device's host in the browser tab title.
     useEffect(() => {
       const active = devices.find((d) => d.id === activeDeviceId);
       document.title = active ? getDeviceHostLabel(active) : "DropVoice";
     }, [activeDeviceId, devices]);
   ```

9. **`handleAddDevice`.** Change:
   ```ts
     const handleAddDevice = useCallback(
       (url: string, name?: string) => {
         return addDevice(url, name);
       },
       [addDevice],
     );
   ```
   →
   ```ts
     const handleAddDevice = useCallback(
       (url: string) => {
         return addDevice(url);
       },
       [addDevice],
     );
   ```

10. **`handleRenameDevice` / `handleSaveRename`.** Delete both callbacks entirely:
    ```ts
        const handleRenameDevice = useCallback((deviceId: string) => {
          setRenameDeviceId(deviceId);
          setShowRename(true);
        }, []);

        const handleSaveRename = useCallback(
          (newName: string) => {
            if (!renameDeviceId) return;
            renameDevice(renameDeviceId, newName);
          },
          [renameDevice, renameDeviceId],
        );
    ```

11. **`handleConfirmRemove`.** Remove the rename-cleanup branch. Replace the whole callback:
    ```ts
        const handleConfirmRemove = useCallback(() => {
          if (!removeDeviceId) return;
          removeDevice(removeDeviceId);
          if (renameDeviceId === removeDeviceId) {
            setRenameDeviceId(null);
            setShowRename(false);
          }
          setRemoveDeviceId(null);
        }, [removeDevice, removeDeviceId, renameDeviceId]);
    ```
    →
    ```ts
        const handleConfirmRemove = useCallback(() => {
          if (!removeDeviceId) return;
          removeDevice(removeDeviceId);
          setRemoveDeviceId(null);
        }, [removeDevice, removeDeviceId]);
    ```

12. **Active-device helper.** Add (near the other `useMemo`s, before `return`):
    ```ts
        const activeDevice = useMemo(
          () => devices.find((d) => d.id === activeDeviceId) ?? null,
          [activeDeviceId, devices],
        );
        const dark = resolvedTheme === "dark";
    ```

13. **JSX — replace `<DeviceTabs …/>` with the slim top row.** Replace:
    ```tsx
          <DeviceTabs
            devices={devices}
            activeDeviceId={activeDeviceId}
            onSelect={setActiveDevice}
            onRemove={handleRemoveDevice}
            onRename={handleRenameDevice}
            onRetry={retryDevice}
            onAdd={() => setShowAddDevice(true)}
          />
    ```
    →
    ```tsx
          <div className="flex min-h-[2.25rem] items-center justify-between">
            <ConnectionPill device={activeDevice} onRetry={() => activeDevice && retryDevice(activeDevice.id)} />
            <DeviceMenu
              canRemove={!!activeDeviceId}
              onAdd={() => setShowAddDevice(true)}
              onRemove={() => activeDeviceId && setRemoveDeviceId(activeDeviceId)}
            />
          </div>
    ```
    (`handleRemoveDevice` callback is now unused — delete it too: remove the `handleRemoveDevice` `useCallback` that just calls `setRemoveDeviceId`. The kebab wires `onRemove` directly.)

14. **JSX — remove swipe handlers from the action row.** Replace:
    ```tsx
              <div
                className="mt-6 flex items-center justify-between px-1"
                onTouchStart={handleTouchStart}
                onTouchEnd={handleTouchEnd}
              >
    ```
    →
    ```tsx
              <div className="mt-6 flex items-center justify-between px-1">
    ```

15. **JSX — add the dot row + empty hint.** Immediately after the closing `</TextInput>` usage and before the action-row `<div>`, insert:
    ```tsx
              {devices.length === 0 ? (
                <p className="mt-3 text-center text-sm text-muted-foreground dark:text-slate-400">
                  {t("devices.emptyState")}
                </p>
              ) : (
                <DeviceDots
                  devices={devices}
                  activeDeviceId={activeDeviceId}
                  dark={dark}
                  onSelect={setActiveDevice}
                />
              )}
    ```

16. **JSX — remove `<DeviceNameEditor …/>`.** Delete the entire block:
    ```tsx
          <DeviceNameEditor
            isOpen={showRename}
            currentName={devices.find((d) => d.id === renameDeviceId)?.name ?? ""}
            onSave={handleSaveRename}
            onClose={() => {
              setShowRename(false);
              setRenameDeviceId(null);
            }}
          />
    ```

17. **JSX — ConfirmDialog message uses IP.** Replace:
    ```tsx
            message={t("devices.removeConfirm", {
              name: devices.find((d) => d.id === removeDeviceId)?.name ?? "",
            })}
    ```
    →
    ```tsx
            message={t("devices.removeConfirm", {
              ip: getDeviceHostLabel(
                devices.find((d) => d.id === removeDeviceId) ?? { id: "", name: "", url: "", status: "disconnected", lastConnected: 0 },
              ),
            })}
    ```
    (The fallback object satisfies the `Device` shape for `getDeviceHostLabel` when `removeDeviceId` is null between interactions.)

### Step 4: Delete the old components

Run:
```bash
git rm src/mobile/components/DeviceTabs.tsx src/mobile/components/DeviceCard.tsx src/mobile/components/DeviceNameEditor.tsx
```

### Step 5: Type-check + build

Run: `pnpm build`
Expected: `tsc` passes with no errors (watch for any leftover unused imports — `handleRemoveDevice`, `renameDevice`, `DeviceTabs`, `DeviceNameEditor`, `touchStartRef` must all be gone) and Vite produces the bundle.

If `tsc` reports an unused symbol, delete that import/symbol and re-run.

### Step 6: Commit

```bash
git add -A
git commit -m "feat(mobile): replace device tabs with IP-colored dot switcher and kebab menu"
```

---

## Task 7: Locale updates (en, zh, zh-TW, ja)

Remove rename/name keys, switch Remove confirm + switch toast to IP, add kebab keys, rephrase empty state.

**Files:**
- Modify: `src/mobile/locales/en.json`
- Modify: `src/mobile/locales/zh.json`
- Modify: `src/mobile/locales/zh-TW.json`
- Modify: `src/mobile/locales/ja.json`

In **each** of the four files, within the `"devices"` object:

1. Delete these keys: `"deviceNameOptional"`, `"namePlaceholder"`, `"renameDevice"`, `"renameHint"`.
2. Change `"removeConfirm"` to use `{{ip}}` instead of `{{name}}`.
3. Change `"switchedTo"` to use `{{ip}}` instead of `{{name}}`.
4. Add two keys: `"menu"` and `"removeDevice"`.
5. Rephrase `"emptyState"`.

Exact values per file:

**en.json**
```json
    "emptyState": "No device yet. Open the menu to add one.",
    "menu": "Menu",
    "removeDevice": "Remove device",
    "removeConfirm": "Remove {{ip}}?",
    "switchedTo": "Switched to {{ip}}",
```

**zh.json**
```json
    "emptyState": "暂无设备，请打开菜单添加",
    "menu": "菜单",
    "removeDevice": "移除设备",
    "removeConfirm": "移除 {{ip}}？",
    "switchedTo": "已切换到 {{ip}}",
```

**zh-TW.json**
```json
    "emptyState": "尚無裝置，請開啟選單新增",
    "menu": "選單",
    "removeDevice": "移除裝置",
    "removeConfirm": "移除 {{ip}}？",
    "switchedTo": "已切換到 {{ip}}",
```

**ja.json**
```json
    "emptyState": "デバイスがありません。メニューから追加してください。",
    "menu": "メニュー",
    "removeDevice": "デバイスを削除",
    "removeConfirm": "{{ip}}を削除しますか？",
    "switchedTo": "{{ip}}に切り替えました",
```

**Step: Validate JSON + build**

Run: `pnpm build`
Expected: passes (valid JSON; i18next resolves the new keys). If a `tsc`/build error mentions a locale, fix the JSON syntax (trailing comma, missing comma) and re-run.

**Commit:**
```bash
git add src/mobile/locales
git commit -m "i18n(mobile): drop rename keys, switch remove/switch to IP, add menu keys"
```

---

## Task 8: Final verification

**Step 1: Full build + unit tests**

Run: `pnpm build && pnpm test`
Expected: both pass.

**Step 2: Run the mobile page**

- Frontend-only visual check: `pnpm dev`, then open `http://localhost:5173/src/mobile/index.html` in a mobile-width browser.
- Full connected check (to see "connected" dots + real send): `pnpm tauri dev` and scan the QR from a phone on the same LAN — or open the served mobile URL directly.

**Step 3: Visual checklist**

Confirm each against the design:
- [ ] **Single device:** no dot row rendered.
- [ ] **Two+ devices:** centered dots; active = solid + visibly larger; others = hollow + smaller; each a distinct color; colors stable across re-renders.
- [ ] **Switch:** tapping an inactive dot makes it active (solid/large); `document.title` updates to that device's IP/host; the previously-active dot becomes hollow/smaller.
- [ ] **Ordering stable:** switching devices does not reorder the dots; adding a device appends a dot on the right.
- [ ] **Active disconnecting:** active dot pulses; (with backend stopped) the red pill `⚠ <error> · Retry` appears above the textarea; Send is disabled; Retry re-attempts.
- [ ] **Inactive disconnecting/connecting:** those dots pulse / dim respectively; their IP hue is still recognizable.
- [ ] **Kebab:** `⋮` opens a menu with *Add device* and *Remove device*; clicking outside closes it.
- [ ] **Remove:** *Remove device* opens the confirm dialog showing the active device's IP; confirm removes it and the dot disappears; cancel does nothing.
- [ ] **Add:** *Add device* opens the modal with QR/URL tabs and **no** name field; adding appends a new dot.
- [ ] **Dark mode:** toggle theme in Settings; dot colors visibly shift to the darker-theme S/L; pill and menu remain legible.
- [ ] **No rename UI anywhere;** no dead "PC-xxx" rename prompts; long-pressing a dot does nothing special (no rename).

**Step 4: Fix anything broken; commit**

If the checklist surfaces issues, fix them, re-run `pnpm build`, then:
```bash
git add -A
git commit -m "fix(mobile): polish dot switcher per verification"
```
If all clean, no commit needed.

---

## Notes for the executor

- **Strict TS cleanup is the main risk.** After Task 6, search the repo for any remaining references to `DeviceTabs`, `DeviceCard`, `DeviceNameEditor`, `renameDevice`, `handleRename`, `touchStart`, `handleTouchStart`, `handleTouchEnd`, `handleRemoveDevice`, `nameInput`, `deviceNameOptional`, `namePlaceholder`, `renameHint` — all must be gone or `pnpm build` will fail.
- **`getDefaultDeviceName` is still used** by `useMultiWebSocket.addDevice` and `useDeviceManager` — keep it in `deviceStorage.ts`.
- **`ConfirmDialog` is still used** (Remove flow) — keep it.
- **Mobile `index.html` title** is the initial value before the `document.title` effect runs; it's fine for it to say "DropVoice".
- **No new dependencies beyond vitest.** `lucide-react` (already present) supplies `MoreVertical`, `Plus`, `Trash2`.
