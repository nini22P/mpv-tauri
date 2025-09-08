import { initializeMpv, MpvConfig } from 'tauri-plugin-mpv-api';

const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

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

try {
  console.log('Initializing mpv with properties:', OBSERVED_PROPERTIES);
  await initializeMpv(mpvConfig);
  console.log('mpv initialization completed successfully!');
} catch (error) {
  console.error('mpv initialization failed:', error);
}