# Tauri MPV Plugin

A Tauri plugin that embeds MPV media player window into your Tauri applications.

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
import { initializeMpv, listenMpvEvents, sendMpvCommand, setVideoMarginRatio } from 'tauri-plugin-mpv-api';
import { getCurrentWindow } from '@tauri-apps/api/window';

const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

// Initialize MPV
await initializeMpv({
  observedProperties: OBSERVED_PROPERTIES,
  mpvConfig: {
    'vo': 'gpu',
    'hwdec': 'auto',
    'media-controls': 'no',
  }
});

// Listen events
const unlisten = await listenMpvEvents<typeof OBSERVED_PROPERTIES[number]>((event) => {
  if (event.event === 'property-change') {
    console.log('Pause state changed:', mpvEvent.name, mpvEvent.data);
  }
});

// Load and play a file
await sendMpvCommand({ command: ['loadfile', '/path/to/video.mp4'] });
await sendMpvCommand({ command: ['set_property', 'pause', false] });

// Set video margin ratio
await setVideoMarginRatio({ left: 0.05, right: 0.05, top: 0.05, bottom: 0.05 });
```

## Examples

See the [examples](./examples) directory for complete working examples.

## Platform Support

- ✅ **Windows** - Fully tested and supported
- ⚠️ **Linux** - Not test
- ⚠️ **macOS** - Not test

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
