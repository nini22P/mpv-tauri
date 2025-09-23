# Tauri Plugin libmpv

A Tauri plugin to embed the mpv player in your app via libmpv.

## Installation

### Prerequisites

- Download the libmpv2 library and place it in `src-tauri`.
- Tauri v2.x
- Node.js 18+

### Install the Plugin

```bash
npm run tauri add libmpv
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
import { destroy, init, MpvConfig, observeProperties, command, MpvObservableProperty } from 'tauri-plugin-libmpv-api';

// Properties to observe
const OBSERVED_PROPERTIES = [
  ['pause', 'flag'],
  ['time-pos', 'double'],
  ['duration', 'double'],
  ['filename', 'string'],
] as const satisfies MpvObservableProperty[];

// mpv configuration
const mpvConfig: MpvConfig = {
  initialProperties: {
    'vo': 'gpu-next',
    'hwdec': 'auto-safe',
    'keep-open': 'yes',
    'force-window': 'yes',
  },
  initialProperties: {
    'pause': 'yes',
  },
  observedProperties: OBSERVED_PROPERTIES,
};

// Initialize mpv
try {
  console.log('Initializing mpv with properties:', OBSERVED_PROPERTIES);
  await init(mpvConfig);
  console.log('mpv initialization completed successfully!');
} catch (error) {
  console.error('mpv initialization failed:', error);
}

// Observe properties
const unlisten = await observeProperties(
  OBSERVED_PROPERTIES,
  ({ name, change }) => {
    switch (name) {
      case 'pause':
        console.log('Playback paused state:', change);
        break;
      case 'time-pos':
        console.log('Current time position:', change);
        break;
      case 'duration':
        console.log('Duration:', change);
        break;
      case 'filename':
        console.log('Current playing file:', change);
        break;
    }
  });

// Unlisten when no longer needed
unlisten();

// Load and play a file
await command('loadfile', ['/path/to/video.mp4']);

// Destroy mpv when no longer needed
await destroy();

```

## Platform Support

- ✅ **Windows** - Fully tested and supported
- ⚠️ **Linux** - Not tested
- ⚠️ **macOS** - Not tested

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the LGPL-2.1 License - see the [LICENSE](LICENSE) file for details.
