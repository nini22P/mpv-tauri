import { listenMpvEvents } from 'tauri-plugin-mpv-api';

const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;

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
unlisten();