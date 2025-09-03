import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { COMMON_PROPERTIES, initializeMpv, sendMpvCommand, listenMpvEvents } from 'tauri-plugin-mpv-api';

const OBSERVED_PROPERTIES = COMMON_PROPERTIES;

interface PlaylistItem {
  current?: boolean;
  filename: string;
  id: number;
  playing?: boolean;
  'playlist-path'?: string;
}

export type Connection = 'pending' | 'connected' | 'error';

interface PlayerState {
  connection: Connection;
  isPaused: boolean;
  playlist: PlaylistItem[];
  currentFile: string | null;
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
  const [state, setState] = useState<PlayerState>({
    connection: 'pending',
    isPaused: true,
    currentFile: null,
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
    const initMpv = async () => {
      try {
        console.log('🎬 Initializing MPV with properties:', OBSERVED_PROPERTIES);
        await initializeMpv({
          observedProperties: Array.from(OBSERVED_PROPERTIES),
          mpvConfig: {
            'vo': 'gpu-next',
            'hwdec': 'auto',
            'media-controls': 'no',
          }
        });
        console.log('🎬 MPV initialization completed successfully!');
      } catch (error) {
        console.error('🎬 MPV initialization failed:', error);
        setState(prev => ({ ...prev, connection: 'error' }));
      }
    };

    initMpv();
  }, [])

  useEffect(() => {
    let unlistenEvent = listenMpvEvents<typeof OBSERVED_PROPERTIES[number]>((mpvEvent) => {
      const { event } = mpvEvent;

      setState(prev => {
        const newStatus = { ...prev };

        if (newStatus.connection !== 'connected') {
          newStatus.connection = 'connected';
        }

        switch (event) {
          case 'property-change':
            const { name, data } = mpvEvent
            switch (name) {
              case 'playlist':
                newStatus.playlist = Array.isArray(data) ? data : [];
                break;
              case 'filename':
                newStatus.currentFile = typeof data === 'string' ? data : null;
                break;
              case 'pause':
                newStatus.isPaused = typeof data === 'boolean' ? data : true;
                break;
              case 'eof-reached':
                newStatus.eofReached = typeof data === 'boolean' ? data : false;
                break;
              case 'time-pos':
                newStatus.timePos = typeof data === 'number' ? data : newStatus.timePos;
                break;
              case 'duration':
                newStatus.duration = typeof data === 'number' ? data : newStatus.duration;
                break;
              case 'volume':
                newStatus.volume = typeof data === 'number' ? data : newStatus.volume;
                break;
              case 'mute':
                newStatus.mute = typeof data === 'boolean' ? data : newStatus.mute;
                break;
              case 'speed':
                newStatus.speed = typeof data === 'number' ? data : newStatus.speed;
                break;
              case 'percent-pos':
                newStatus.percentPos = typeof data === 'number' ? data : newStatus.percentPos;
                break;
              default:
                break;
            }
            break;
          case 'end-file':
            newStatus.eofReached = true;
            newStatus.currentFile = null;
            newStatus.timePos = 0;
            newStatus.duration = 0;
            break;
          default:
            break;
        }

        return newStatus;
      });
    })

    return () => {
      unlistenEvent.then(unlistenFn => unlistenFn());
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
    if (state.currentFile && (state.duration - state.timePos < 1 || state.eofReached)) {
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