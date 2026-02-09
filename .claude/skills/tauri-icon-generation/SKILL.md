---
name: tauri-icon-generation
description: Use when working with Tauri projects that need app icons generated for desktop (Windows, macOS, Linux) or mobile platforms (iOS, Android). Use when you see a source icon file (icon.png, app-icon.png) that needs to be converted into platform-specific formats.
---

# Tauri Icon Generation

## Overview

Tauri provides a built-in CLI command that automatically generates all required icon formats for desktop and mobile platforms from a single source image. This is the standard and recommended approach - no manual icon creation or third-party tools needed.

## Core Principle

**One source image → All platform icons**

The `pnpm tauri icon` command takes a single source PNG/SVG and generates everything automatically.

## When to Use

- You have a source icon file that needs to be converted for Tauri
- Starting a new Tauri project with custom branding
- Updating app icons across all platforms
- Need icons for desktop, iOS, or Android

## Quick Reference

| Scenario                       | Command                             |
| ------------------------------ | ----------------------------------- |
| Default (using `app-icon.png`) | `pnpm tauri icon`                   |
| Custom source file             | `pnpm tauri icon path/to/icon.png`  |
| Custom output directory        | `pnpm tauri icon -o path/to/output` |
| Specific PNG sizes only        | `pnpm tauri icon -p 512,1024`       |

## Implementation

### Prerequisites

1. **Source image requirements:**
   - Format: PNG or SVG with transparency
   - Shape: Square (width == height)
   - Recommended size: 1024x1024 or higher
   - Color: RGBA (32-bit, 8-bit per channel)

2. **Default source location:** `./app-icon.png` in project root

### Basic Usage

```bash
# Step 1: Ensure your source icon is named app-icon.png
# (or specify the path explicitly)

# Step 2: Run the icon generation command
pnpm tauri icon

# That's it! All icons are generated automatically
```

### With Custom Source File

```bash
# If your icon has a different name
pnpm tauri icon my-custom-icon.png

# The command will use your file as input
```

### What Gets Generated

**Desktop icons** (placed in `src-tauri/icons/`):

- Windows: `icon.ico` (multi-layer: 16, 24, 32, 48, 64, 256px)
- macOS: `icon.icns` (all required sizes)
- PNG: `32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, `icon.png`
- Windows Store: Various square sizes (30x30 through 310x310)

**Mobile icons** (placed in `src-tauri/icons/android/` and `src-tauri/icons/ios/`):

- Android: Complete set across 5 densities (hdpi through xxxhdpi)
- iOS: Complete set with all required sizes and scales

### Integration

**Desktop:** Icons are automatically included in your built app. The `tauri.conf.json` already references the correct paths:

```json
{
  "bundle": {
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**Mobile:** Copy generated icons to your mobile projects:

- Android: Copy from `src-tauri/icons/android/` to your Android project's res folders
- iOS: Copy from `src-tauri/icons/ios/` to `src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/`

## Common Mistakes

| Mistake                                                       | Fix                                                                  |
| ------------------------------------------------------------- | -------------------------------------------------------------------- |
| Source image is `icon.png` but command expects `app-icon.png` | Rename to `app-icon.png` or specify path: `pnpm tauri icon icon.png` |
| Using low-resolution source image                             | Use 1024x1024 or higher for best quality                             |
| Manually creating icons with image tools                      | Use the CLI - it's faster and guaranteed correct                     |
| Forgetting to regenerate after updating source                | Re-run `pnpm tauri icon` after source changes                        |
| Trying to generate iOS/Android icons manually                 | CLI handles this automatically                                       |

## Verification

After running the command:

```bash
# Check desktop icons were created
ls -la src-tauri/icons/

# Verify key files exist
ls src-tauri/icons/icon.ico
ls src-tauri/icons/icon.icns
ls src-tauri/icons/icon.png

# Check mobile icons
ls src-tauri/icons/android/mipmap-*/ic_launcher.png
ls src-tauri/icons/ios/AppIcon-*.png
```

## Platform-Specific Notes

### Windows

- The `.ico` file includes multiple layers for different display contexts
- StoreLogo variants are automatically generated for Microsoft Store submission

### macOS

- The `.icns` file contains all required retina and standard resolutions
- No manual editing needed

### iOS

- Icons are generated **without transparency** (iOS requirement)
- All sizes and scaling factors are handled automatically

### Android

- Adaptive icons with foreground layers
- Round and standard variants for all density buckets (hdpi through xxxhdpi)

## Real-World Impact

Before this command: Manually creating icons using image editors, guessing at platform requirements, missing sizes, incorrect formats - hours of work with inconsistent results.

After this command: Single command, 30 seconds, perfect icons for all platforms every time.
