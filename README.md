# Tauri Plugin MPV

A Tauri plugin that embeds MPV media player window into your Tauri applications.

This plugin provides a simple and efficient way to embed MPV into your Tauri applications. It uses MPV's JSON IPC interface to communicate with the MPV process.

## Installation

### Prerequisites

- [MPV](https://mpv.io/) must be installed and available in your system PATH
- Tauri v2.x
- Node.js 18+

### Install the Plugin

```bash
npm run tauri add mpv
```

### Configure Window Transparency

For MPV to properly embed into your Tauri window, you need to configure transparency:

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

// MPV configuration
const mpvConfig: MpvConfig = {
  mpvArgs: [
    '--no-config',
    '--vo=gpu-next',
    '--hwdec=auto-safe',
    '--media-controls=no',
  ],
  observedProperties: OBSERVED_PROPERTIES,
  ipcTimeoutMs: 2000,
  showMpvOutput: false,
};

// Initialize MPV
try {
  console.log('Initializing MPV with properties:', OBSERVED_PROPERTIES);
  await initializeMpv(mpvConfig);
  console.log('MPV initialization completed successfully!');
} catch (error) {
  console.error('MPV initialization failed:', error);
}

// Destroy MPV when no longer needed
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
