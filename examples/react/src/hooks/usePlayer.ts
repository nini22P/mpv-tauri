import { useEffect, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { initializeMpv, destroyMpv, sendMpvCommand, MpvPlaylistItem, observeMpvProperties, MpvConfig } from 'tauri-plugin-mpv-api';

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

export type Connection = 'pending' | 'connected' | 'error';

interface PlayerState {
  connection: Connection;
  isPaused: boolean;
  playlist: MpvPlaylistItem[];
  currentFile?: string;
  eofReached: boolean;
  timePos: number;
  duration: number;
  isFullscreen: boolean;
  volume?: number;
  mute?: boolean;
  speed?: number;
  percentPos?: number;
}

interface PlayerActions {
  loadFile: (file: string) => Promise<void>;
  playlistPlay: (id: number) => Promise<void>;
  playlistNext: () => Promise<void>;
  playlistPrev: () => Promise<void>;
  play: () => Promise<void>;
  pause: () => Promise<void>;
  stop: () => Promise<void>;
  seek: (value: number) => Promise<void>;
  seekForward: () => Promise<void>;
  seekBackward: () => Promise<void>;
  toggleFullscreen: () => Promise<void>;
}

export type Player = PlayerState & PlayerActions;

const usePlayer = (): Player => {

  const tolerance = 0.5

  const lastUpdateTime = useRef(0);

  const [state, setState] = useState<PlayerState>({
    connection: 'pending',
    isPaused: true,
    currentFile: undefined,
    playlist: [],
    eofReached: false,
    timePos: 0,
    duration: 0,
    isFullscreen: false,
    volume: 100,
    mute: false,
    speed: 1.0,
    percentPos: 0,
  });

  useEffect(() => {
    (async () => {

      const mpvConfig: MpvConfig = {
        mpvArgs: [
          '--vo=gpu-next',
          '--hwdec=auto-safe',
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
        setState(prev => ({ ...prev, connection: 'error' }));
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
        if (name === 'time-pos') {
          const now = Date.now();
          if (now - lastUpdateTime.current < 250) {
            return;
          }
          lastUpdateTime.current = now;
        }

        setState(prev => {
          const newStatus = { ...prev };

          if (newStatus.connection !== 'connected') {
            newStatus.connection = 'connected';
          }

          switch (name) {
            case 'playlist':
              newStatus.playlist = data;
              break;
            case 'filename':
              newStatus.currentFile = data;
              break;
            case 'pause':
              newStatus.isPaused = data;
              break;
            case 'eof-reached':
              newStatus.eofReached = data ?? false;
              break;
            case 'time-pos':
              newStatus.timePos = data ?? 0;
              break;
            case 'duration':
              newStatus.duration = data ?? 0;
              break;
            case 'volume':
              newStatus.volume = data;
              break;
            case 'mute':
              newStatus.mute = data;
              break;
            case 'speed':
              newStatus.speed = data;
              break;
            default:
              console.log(name, data);
              break;
          }

          return newStatus;
        });
      });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);

  const loadFile = async (file: string) => {
    await sendMpvCommand({ command: ['loadfile', file] });
    await sendMpvCommand({ command: ['set_property', 'pause', false] });
  };

  const playlistPlay = async (index: number) => {
    await sendMpvCommand({ command: ['playlist-play-index', index] });
  };

  const playlistNext = async () => {
    await sendMpvCommand({ command: ['playlist-next'] });
  };

  const playlistPrev = async () => {
    await sendMpvCommand({ command: ['playlist-prev'] });
  };

  const play = async () => {
    if (state.currentFile && (state.duration - state.timePos < tolerance || state.eofReached)) {
      await seek(0);
    }
    await sendMpvCommand({ command: ['set_property', 'pause', false] });
  };

  const pause = async () => {
    await sendMpvCommand({ command: ['set_property', 'pause', true] });
  };

  const stop = async () => {
    await pause();
    await sendMpvCommand({ command: ['stop'] });
  };

  const seek = async (value: number) => {
    await sendMpvCommand({ command: ['seek', value, 'absolute'] });
  };

  const seekForward = async () => {
    await sendMpvCommand({ command: ['seek', '10', 'relative'] });
  };

  const seekBackward = async () => {
    await sendMpvCommand({ command: ['seek', '-10', 'relative'] });
  };

  const toggleFullscreen = async () => {
    await getCurrentWindow().setFullscreen(!state.isFullscreen);
    setState(prev => ({ ...prev, isFullscreen: !prev.isFullscreen }));
  };

  return {
    ...state,
    isPaused: state.isPaused || !state.currentFile,
    timePos: state.duration - state.timePos < tolerance ? state.duration : state.timePos,
    loadFile,
    playlistPlay,
    playlistNext,
    playlistPrev,
    play,
    pause,
    stop,
    seek,
    seekForward,
    seekBackward,
    toggleFullscreen,
  };
}

export default usePlayer;