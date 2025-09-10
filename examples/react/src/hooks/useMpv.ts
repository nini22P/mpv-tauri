import { useEffect } from 'react';
import { initializeMpv, observeMpvProperties, MpvConfig, destroyMpv } from 'tauri-plugin-mpv-api';
import usePlayerStore from '../store';

const OBSERVED_PROPERTIES = [
  'playlist',
  'filename',
  'pause',
  'eof-reached',
  'time-pos',
  'duration',
  'volume',
  'mute',
  'speed',
] as const;

const useMpv = () => {

  const connection = usePlayerStore.use.connection();

  const setConnection = usePlayerStore.use.setConnection();
  const setIsPaused = usePlayerStore.use.setIsPaused();
  const setCurrentFile = usePlayerStore.use.setCurrentFile();
  const setPlaylist = usePlayerStore.use.setPlaylist();
  const setEofReached = usePlayerStore.use.setEofReached();
  const setTimePos = usePlayerStore.use.setTimePos();
  const setDuration = usePlayerStore.use.setDuration();
  const setVolume = usePlayerStore.use.setVolume();
  const setMute = usePlayerStore.use.setMute();
  const setSpeed = usePlayerStore.use.setSpeed();

  useEffect(() => {
    (async () => {
      const mpvConfig: MpvConfig = {
        mpvArgs: [
          '--vo=gpu-next',
          '--hwdec=auto-safe',
          '--keep-open=yes',
          '--force-window',
          '--pause',
        ],
        observedProperties: OBSERVED_PROPERTIES,
        ipcTimeoutMs: 2000,
      };

      try {
        console.log('Initializing mpv with properties:', OBSERVED_PROPERTIES);
        await initializeMpv(mpvConfig);
        console.log('mpv initialization completed successfully!');
        setConnection('connected');
      } catch (error) {
        console.error('mpv initialization failed:', error);
        setConnection('error');
      }
    })();
  }, [])

  useEffect(() => {
    const handleBeforeUnload = (_event: BeforeUnloadEvent) => destroyMpv();
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  }, []);

  useEffect(() => {
    let unlistenPromise = observeMpvProperties(
      OBSERVED_PROPERTIES,
      ({ name, data }) => {
        if (connection !== 'connected')
          setConnection('connected');
        switch (name) {
          case 'playlist':
            setPlaylist(data);
            break;
          case 'filename':
            setCurrentFile(data);
            break;
          case 'pause':
            setIsPaused(data);
            break;
          case 'eof-reached':
            setEofReached(data ?? false);
            break;
          case 'time-pos':
            setTimePos(data ?? 0);
            break;
          case 'duration':
            setDuration(data ?? 0);
            break;
          case 'volume':
            setVolume(data);
            break;
          case 'mute':
            setMute(data);
            break;
          case 'speed':
            setSpeed(data);
            break;
          default:
            console.log(name, data);
            break;
        }
      });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);
}

export default useMpv;