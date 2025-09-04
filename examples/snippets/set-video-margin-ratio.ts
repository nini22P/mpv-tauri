import { setVideoMarginRatio } from 'tauri-plugin-mpv-api';

// Leave 10% space at bottom for control bar
await setVideoMarginRatio({ bottom: 0.1 });

// Leave margins on all sides
await setVideoMarginRatio({
  left: 0.05,
  right: 0.05,
  top: 0.1,
  bottom: 0.15
});

// Reset margins (remove all margins)
await setVideoMarginRatio({
  left: 0,
  right: 0,
  top: 0,
  bottom: 0
});