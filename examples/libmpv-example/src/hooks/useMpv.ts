import { useEffect } from 'react';
import { init, observeProperties, MpvConfig, destroy, MpvObservableProperty, listenEvents } from 'tauri-plugin-libmpv-api';
import usePlayerStore from '../store';

const OBSERVED_PROPERTIES = [
  ['playlist', 'string'],
  ['filename', 'string'],
  ['pause', 'flag'],
  ['eof-reached', 'flag'],
  ['time-pos', 'double'],
  ['duration', 'double'],
  ['volume', 'double'],
  ['mute', 'flag'],
  ['speed', 'double'],
] as const satisfies MpvObservableProperty[];

const useMpv = () => {

  const updatePlayerState = usePlayerStore.use.updatePlayerState();

  useEffect(() => {
    (async () => {
      const mpvConfig: MpvConfig = {
        initialProperties: {
          'vo': 'gpu-next',
          'hwdec': 'auto-safe',
          'keep-open': 'yes',
          'force-window': 'yes',
          'pause': 'yes',
        },
        observedProperties: OBSERVED_PROPERTIES,
      };

      try {
        console.log('Initializing mpv with properties:', OBSERVED_PROPERTIES);
        await init(mpvConfig);
        console.log('mpv initialization completed successfully!');
        updatePlayerState('connection', 'connected');
      } catch (error) {
        console.error('mpv initialization failed:', error);
        updatePlayerState('connection', 'error');
      }
    })();
  }, [])

  useEffect(() => {
    const handleBeforeUnload = (_event: BeforeUnloadEvent) => destroy();
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  }, []);

  useEffect(() => {
    let unlistenPromise = listenEvents(
      (mpvEvent) => {
        if (mpvEvent.event !== 'property-change') {
          console.log(mpvEvent);
        }
      });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);

  useEffect(() => {
    let unlistenPromise = observeProperties(
      OBSERVED_PROPERTIES,
      ({ name, change }) => {
        if (name !== 'time-pos')
          console.log(name, change)
        switch (name) {
          case 'playlist':
            updatePlayerState('playlist', JSON.parse(change));
            break;
          case 'filename':
            updatePlayerState('filename', change);
            break;
          case 'pause':
            updatePlayerState('isPaused', change);
            break;
          case 'eof-reached':
            updatePlayerState('eofReached', change ?? false);
            break;
          case 'time-pos':
            updatePlayerState('timePos', change ?? 0);
            break;
          case 'duration':
            updatePlayerState('duration', change ?? 0);
            break;
          case 'volume':
            updatePlayerState('volume', change);
            break;
          case 'mute':
            updatePlayerState('mute', change);
            break;
          case 'speed':
            updatePlayerState('speed', change);
            break;
          default:
            console.log(name, change);
            break;
        }
      });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);
}

export default useMpv;