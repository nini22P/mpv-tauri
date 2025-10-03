# Tauri Plugin libmpv

A Tauri plugin for embedding the mpv player in your app via libmpv.

## Installation

### Prerequisites

- Setup libmpv development environment.
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
  initialOptions: {
    'vo': 'gpu-next',
    'hwdec': 'auto-safe',
    'keep-open': 'yes',
    'force-window': 'yes',
  },
  observedProperties: OBSERVED_PROPERTIES,
};

// Initialize mpv
try {
  await init(mpvConfig);
  console.log('mpv initialization completed successfully!');
} catch (error) {
  console.error('mpv initialization failed:', error);
}

// Observe properties
const unlisten = await observeProperties(
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

This project is licensed under the MPL-2.0 License - see the [LICENSE](LICENSE) file for details.
