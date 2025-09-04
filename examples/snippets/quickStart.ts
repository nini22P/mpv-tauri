import { initializeMpv, listenMpvEvents, sendMpvCommand } from 'tauri-plugin-mpv-api';

// Properties to observe
const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

// Initialize MPV
try {
  console.log('🎬 Initializing MPV with properties:', OBSERVED_PROPERTIES);
  await initializeMpv({
    observedProperties: OBSERVED_PROPERTIES,
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

// Listen events
const unlisten = await listenMpvEvents<typeof OBSERVED_PROPERTIES[number]>((mpvEvent) => {
  if (mpvEvent.event === 'property-change') {
    switch (mpvEvent.name) {
      case 'pause':
        console.log('Playback paused state:', mpvEvent.data);
        break;
      case 'time-pos':
        // console.log('Current time position:', mpvEvent.data);
        break;
      case 'duration':
        if (typeof mpvEvent.data === 'number' && mpvEvent.data > 0) {
          console.log('File is ready to play. Duration:', mpvEvent.data);
        }
        break;
      case 'filename':
        console.log('Current playing file:', mpvEvent.data);
        break;
    }
  }
});

// Unlisten events when no longer needed
// unlisten();

// Load and play a file
await sendMpvCommand({ command: ['loadfile', '/path/to/video.mp4'] });
