# Tauri Plugin mpv

A Tauri plugin for embedding the mpv player in your app via JSON IPC.

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

#### 1. Set window transparency in `tauri.conf.json`

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

#### 2. Set web page background to transparent in your CSS

```css
/* In your main CSS file */
html,
body {
  background: transparent;
}
```

## Quick Start

```typescript
import {
  init,
  destroy,
  command,
  setProperty,
  getProperty,
  observeMpvProperties,
  MpvConfig
} from "tauri-plugin-mpv-api";

// Properties to observe
const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

// mpv configuration
const mpvConfig: MpvConfig = {
  args: [
    '--vo=gpu-next',
    '--hwdec=auto-safe',
    '--keep-open=yes',
    '--force-window',
  ],
  observedProperties: OBSERVED_PROPERTIES,
  ipcTimeoutMs: 2000,
};

try {
  await init(mpvConfig);
  console.log('mpv initialization completed successfully!');
} catch (error) {
  console.error('mpv initialization failed:', error);
}

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

// Use the simple shortcut for most commands
await command('loadfile', ['/path/to/video.mp4']);
await command('seek', [10, 'relative']); // Seek 10 seconds forward

// Use the full object format if you need to provide a custom request_id
await command({ command: ['stop'], request_id: 123 });

// `setProperty` is type-safe for known properties
await setProperty('volume', 75);
await setProperty('pause', false);

// `getProperty` returns a typed value for known properties
const volume = await getProperty('volume'); // `volume` is a number
console.log('Current volume:', volume);

// You can also explicitly set the type for unknown or custom properties
const customProp = await getProperty<string>('my-custom-property');

// Destroy mpv when your app closes or the player is no longer needed
await destroy();

```

## Platform Support

- ✅ **Windows** - Fully tested and supported
- ⚠️ **Linux** - Not tested
- ⚠️ **macOS** - Not tested

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MPL-2.0 License - see the [LICENSE](LICENSE) file for details.
