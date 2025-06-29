import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import sendCommand from '../utils/sendCommand';

type MpvEventType =
  | 'property-change'
  | 'file-loaded'
  | 'end-file';

const OBSERVED_PROPERTIES = [
  'playlist',
  'filename',
  'pause',
  'eof-reached',
  'time-pos',
  'duration',
] as const;

type MpvEventName = typeof OBSERVED_PROPERTIES[number];

interface PlaylistItem {
  current?: boolean;
  filename: string;
  id: number;
  playing?: boolean;
  'playlist-path'?: string;
}

interface MpvEventPayload {
  event_type: MpvEventType;
  name: MpvEventName | null;
  data: string | number | boolean | PlaylistItem[] | null;
}

interface PlayerState {
  isPaused: boolean;
  playlist: PlaylistItem[];
  currentFile: string | null;
  eofReached: boolean;
  timePos: number;
  duration: number;
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
}

export type Player = PlayerState & PlayerActions;

const usePlayer = (): Player => {
  const [state, setState] = useState<PlayerState>({
    isPaused: true,
    currentFile: null,
    playlist: [],
    eofReached: false,
    timePos: 0,
    duration: 0,
  });

  useEffect(() => {
    const unlistenEvent = listen<MpvEventPayload>('mpv-event', (event) => {
      const { event_type, name, data } = event.payload;

      setState(prev => {
        let newStatus = { ...prev };
        switch (event_type) {
          case 'property-change':
            switch (name) {
              case 'playlist':
                console.log('playlist', data);
                newStatus.playlist = Array.isArray(data) ? data : [];
                break;
              case 'filename':
                console.log('filename', data);
                newStatus.currentFile = typeof data === 'string' ? data : null;
                break;
              case 'pause':
                console.log('pause', data);
                newStatus.isPaused = typeof data === 'boolean' ? data : true;
                break;
              case 'eof-reached':
                console.log('eof-reached', data);
                newStatus.eofReached = typeof data === 'boolean' ? data : false;
                break;
              case 'time-pos':
                newStatus.timePos = typeof data === 'number' ? data : newStatus.timePos;
                break;
              case 'duration':
                newStatus.duration = typeof data === 'number' ? data : newStatus.duration;
                break;
              default:
                console.log('property-change', name, data);
                break;
            }
            break;
          case 'file-loaded':
            console.log('file-loaded', data);
            play();
            break;
          case 'end-file':
            console.log('end-file', data);
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
    });

    return () => {
      unlistenEvent.then(unlistenFn => unlistenFn());
    };
  }, []);

  const loadFile = async (file: string) => {
    await sendCommand({ command: ['loadfile', file] });
  };

  const playlistPlay = async (index: number) => {
    await sendCommand({ command: ['playlist-play-index', index] });
  };

  const playlistNext = async () => {
    await sendCommand({ command: ['playlist-next'] });
  };

  const playlistPrev = async () => {
    await sendCommand({ command: ['playlist-prev'] });
  };

  const play = async () => {
    if (state.currentFile && (state.duration - state.timePos < 1 || state.eofReached)) {
      await seek(0);
    }
    await sendCommand({ command: ['set_property', 'pause', false] });
    setState(prev => ({ ...prev, isPaused: false }));
  };

  const pause = async () => {
    await sendCommand({ command: ['set_property', 'pause', true] });
  };

  const stop = async () => {
    await pause();
    await sendCommand({ command: ['stop'] });
  };

  const seek = async (value: number) => {
    await sendCommand({ command: ['seek', value, 'absolute'] });
  };

  const seekForward = async () => {
    await sendCommand({ command: ['seek', '10', 'relative'] });
  };

  const seekBackward = async () => {
    await sendCommand({ command: ['seek', '-10', 'relative'] });
  };

  return {
    ...state,
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
  };
}

export default usePlayer;