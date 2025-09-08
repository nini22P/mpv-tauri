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
