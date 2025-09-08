# Tauri Plugin mpv

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![npm version](https://img.shields.io/npm/v/tauri-plugin-mpv-api.svg)](https://www.npmjs.com/package/tauri-plugin-mpv-api)
![NPM Downloads](https://img.shields.io/npm/d18m/tauri-plugin-mpv-api)
[![Build Status](https://github.com/nini22P/tauri-plugin-mpv/actions/workflows/ci.yml/badge.svg)](https://github.com/nini22P/tauri-plugin-mpv/actions/workflows/ci.yml)

A Tauri plugin to embed the mpv player in your app via JSON IPC.

## Installation

### Prerequisites

- [mpv](https://mpv.io/) must be installed and available in your system PATH
- Tauri v2.x
- Node.js 18+

### Install the Plugin

```bash
npm run tauri add mpv
```

### Configure Window Transparency

For mpv to properly embed into your Tauri window, you need to configure transparency:

#### 1. Set window transparency in `tauri.conf.json`:

```json
{
  "app": {
    "windows": [
      {
        "title": "Your App",
        "width": 1280,
        "height": 720,
        "transparent": true  // Add this line
      }
    ]
  }
}
```

#### 2. Set web page background to transparent in your CSS:

```css
/* In your main CSS file */
html,
body {
  background: transparent;
}
```

## Quick Start

```typescript
import { destroyMpv, initializeMpv, MpvConfig, observeMpvProperties, sendMpvCommand } from "tauri-plugin-mpv-api";

// Properties to observe
const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

// mpv configuration
const mpvConfig: MpvConfig = {
  mpvArgs: [
    '--vo=gpu-next',
    '--hwdec=auto-safe',
    '--keep-open=yes',
    '--force-window',
  ],
  observedProperties: OBSERVED_PROPERTIES,
  ipcTimeoutMs: 2000,
};

// Initialize mpv
try {
  console.log('Initializing mpv with properties:', OBSERVED_PROPERTIES);
  await initializeMpv(mpvConfig);
  console.log('mpv initialization completed successfully!');
} catch (error) {
  console.error('mpv initialization failed:', error);
}

// Destroy mpv when no longer needed
await destroyMpv();

// Observe properties
const unlisten = await observeMpvProperties(
  OBSERVED_PROPERTIES,
  ({ name, data }) => {
    switch (name) {
      case 'pause':
        console.log('Playback paused state:', data);
        break;
      case 'time-pos':
        console.log('Current time position:', data);
        break;
      case 'duration':
        console.log('Duration:', data);
        break;
      case 'filename':
        console.log('Current playing file:', data);
        break;
    }
  });

// Unlisten when no longer needed
unlisten();

// Load and play a file
await sendMpvCommand({ command: ['loadfile', '/path/to/video.mp4'] });

```

## Examples

* [examples/react](./examples/react/): complete working examples.
* [examples/snippets](./examples/snippets/): code snippets.

## Platform Support

- ✅ **Windows** - Fully tested and supported
- ⚠️ **Linux** - Not tested
- ⚠️ **macOS** - Not tested

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
