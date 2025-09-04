import { listenMpvEvents } from 'tauri-plugin-mpv-api';

// Listen events
const unlisten = await listenMpvEvents((mpvEvent) => {
  if (mpvEvent.event === 'property-change') {
    const { name, data } = mpvEvent
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
  }
});

// Unlisten when no longer needed
unlisten();