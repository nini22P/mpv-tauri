# Tauri Plugin MPV

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
import { initializeMpv, observeMpvProperties, sendMpvCommand } from 'tauri-plugin-mpv-api';

// Properties to observe
const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

// Initialize MPV
try {
  console.log('🎬 Initializing MPV with properties:', OBSERVED_PROPERTIES);
  await initializeMpv({
    observedProperties: Array.from(OBSERVED_PROPERTIES),
    mpvConfig: {
      'vo': 'gpu-next',
      'hwdec': 'auto',
      'media-controls': 'no',
    }
  });
  console.log('🎬 MPV initialization completed successfully!');
} catch (error) {
  console.error('🎬 MPV initialization failed:', error);
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
// unlisten();

// Load and play a file
await sendMpvCommand({ command: ['loadfile', '/path/to/video.mp4'] });

```

## Examples

See the [examples/react](./examples/react/) directory for complete working examples.
See the [examples/snippets](./examples/snippets/) directory for code snippets.

## Platform Support

- ✅ **Windows** - Fully tested and supported
- ⚠️ **Linux** - Not tested
- ⚠️ **macOS** - Not tested

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
