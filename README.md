# DropVoice

DropVoice enables sending voice-to-text input from your mobile phone to a PC.

English | [简体中文](README_zh.md)

## Features

- 🖥️ **Cross-Platform**: Windows, macOS, and Linux support
- 📡 **LAN Communication**: QR code connection + instant WebSocket transmission
- 🌐 **Multi-language**: English and Chinese UI
- 🎨 **Multi-Theme**: Light, dark, and system modes
- 📲 **Mobile-Friendly**: Settings page + PWA support

![DropVoice Screenshot](assets/screenshot.png)

## Quick Start

### Installation

Download the latest release for your platform from [Releases](https://github.com/jukanntenn/DropVoice/releases).

### Usage

1. 🚀 **Start DropVoice** on your PC
   - Server starts automatically
   - QR code displayed for connection

2. 📱 **Connect from Mobile**
   - Scan QR code with your phone
   - Mobile browser opens DropVoice interface

3. ⌨️ **Send Text**
   - Type in the mobile input field
   - Press Enter or tap "Send to PC"
   - Text appears at cursor position on your PC

### Linux Requirements

Linux users need `xdotool` for text input:

```bash
# Ubuntu/Debian
sudo apt install xdotool

# Fedora
sudo dnf install xdotool

# Arch Linux
sudo pacman -S xdotool
```

### macOS Requirements

On macOS, after installation:

1. **Allow app to open**: Right-click the app and select "Open" on first launch, or go to System Settings → Privacy & Security → click "Open Anyway"
2. **Grant Accessibility permission**: Go to System Settings → Privacy & Security → Accessibility → add DropVoice to the list
   - Required for keyboard text injection
   - Without this permission, text will not appear on your PC

## Security Considerations

⚠️ **IMPORTANT**: DropVoice is designed for trusted LAN networks only.

- No authentication beyond basic token
- No encryption for WebSocket traffic
- Use only on secure home/office networks

## Troubleshooting

### Mobile cannot connect

- Verify PC and mobile are on the same network
- Check firewall allows port 38425
- Verify server is running

### Text not appearing

- Ensure cursor is in a text input field on PC
- Check WebSocket connection is active

### Chinese input method issues

Some applications have compatibility issues with Chinese input methods:

- **Telegram garbled text**: Text may appear garbled when using Chinese input method
- **Chinese punctuation swallowed**: Chinese punctuation marks may not appear correctly

**Workaround**: Switch to English input method on your PC before sending text from mobile.

## License

MIT License

## Support

For issues and feature requests, please use the [GitHub issue tracker](https://github.com/jukanntenn/DropVoice/issues).
