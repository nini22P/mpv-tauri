import { initializeMpv } from 'tauri-plugin-mpv-api';

const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

try {
  console.log('Initializing MPV with properties:', OBSERVED_PROPERTIES);
  await initializeMpv({
    observedProperties: Array.from(OBSERVED_PROPERTIES),
    mpvConfig: {
      'vo': 'gpu-next',
      'hwdec': 'auto',
      'media-controls': 'no',
    }
  });
  console.log('MPV initialization completed successfully!');
} catch (error) {
  console.error('MPV initialization failed:', error);
}