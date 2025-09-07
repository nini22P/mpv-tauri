import { initializeMpv, MpvConfig } from 'tauri-plugin-mpv-api';

const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

const mpvConfig: MpvConfig = {
  mpvArgs: [
    '--vo=gpu-next',
    '--hwdec=auto-safe',
  ],
  observedProperties: OBSERVED_PROPERTIES,
  ipcTimeoutMs: 2000,
  showMpvOutput: false,
};

try {
  console.log('Initializing MPV with properties:', OBSERVED_PROPERTIES);
  await initializeMpv(mpvConfig);
  console.log('MPV initialization completed successfully!');
} catch (error) {
  console.error('MPV initialization failed:', error);
}