import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import sendCommand from "../utils/sendCommand";
import { getCurrentWindow } from "@tauri-apps/api/window";

export interface PlayerState {
  isPaused: boolean;
  currentFile: string | null;
  eofReached: boolean;
  timePos: number;
  duration: number;
  percentPos: number;
}

export interface PlayerActions {
  loadFile: (file: string) => Promise<void>;
  play: () => Promise<void>;
  pause: () => Promise<void>;
  stop: () => Promise<void>;
  seek: (value: number) => Promise<void>;
  seekForward: () => Promise<void>;
  seekBackward: () => Promise<void>;
}

export type Player = PlayerState & PlayerActions;

type MpvEventType =
  | 'property-change'
  | 'file-loaded'
  | 'end-file';

type MpvEventName =
  | 'filename'
  | 'pause'
  | 'eof-reached'
  | 'time-pos'
  | 'duration'
  | 'percent-pos';

interface MpvEventPayload {
  event_type: MpvEventType;
  name?: MpvEventName;
  data?: any;
}

const usePlayer: () => Player = () => {
  const [state, setState] = useState<PlayerState>({
    isPaused: true,
    currentFile: null,
    eofReached: false,
    timePos: 0,
    duration: 0,
    percentPos: 0,
  });

  useEffect(() => {
    const unlistenEvent = listen<MpvEventPayload>('mpv-event', (event) => {
      const { event_type, name, data } = event.payload;

      setState(prev => {
        let newStatus = { ...prev };
        switch (event_type) {
          case 'property-change':
            switch (name) {
              case 'pause':
                console.log('pause', data);
                if (typeof data === 'boolean') {
                  newStatus.isPaused = data;
                }
                break;
              case 'filename':
                console.log('filename', data);
                newStatus.currentFile = data || null;
                if (data) {
                  newStatus.isPaused = false;
                  newStatus.eofReached = false;
                  newStatus.timePos = 0;
                  newStatus.duration = 0;
                  newStatus.percentPos = 0;
                }
                break;
              case 'eof-reached':
                console.log('eof-reached', data);
                if (typeof data === 'boolean') {
                  newStatus.eofReached = data;
                  if (data) newStatus.isPaused = true;
                }
                break;
              case 'time-pos':
                newStatus.timePos = typeof data === 'number' ? data : newStatus.timePos;
                break;
              case 'duration':
                newStatus.duration = typeof data === 'number' ? data : newStatus.duration;
                break;
              case 'percent-pos':
                newStatus.percentPos = typeof data === 'number' ? data : newStatus.percentPos;
                break;
              default:
                break;
            }
            break;
          case 'file-loaded':
            sendCommand({ command: ['set_property', 'pause', false] });
            newStatus.eofReached = false;
            break;
          case 'end-file':
            newStatus.eofReached = true;
            newStatus.currentFile = null;
            newStatus.timePos = 0;
            newStatus.duration = 0;
            newStatus.percentPos = 0;
            break;
          default:
            break;
        }

        if (newStatus.duration > 0) {
          newStatus.percentPos = (newStatus.timePos / newStatus.duration) * 100;
        } else if (newStatus.timePos === 0 && newStatus.duration === 0) {
          newStatus.percentPos = 0;
        }

        return newStatus;
      });
    });

    return () => {
      unlistenEvent.then(unlistenFn => unlistenFn());
    };
  }, []);

  useEffect(() => {
    getCurrentWindow().setTitle(state.currentFile ? `${state.currentFile} - MPV Tauri` : 'MPV Tauri');
  }, [state.currentFile])

  const loadFile = async (file: string) => {
    await sendCommand({ command: ['loadfile', file] });
  };

  const play = async () => {
    if (state.eofReached && state.currentFile) {
      await seek(0);
    }
    await sendCommand({ command: ['set_property', 'pause', false] });
  };

  const pause = async () => {
    await sendCommand({ command: ['set_property', 'pause', true] });
  };

  const stop = async () => {
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
    play,
    pause,
    stop,
    seek,
    seekForward,
    seekBackward,
  };
}

export default usePlayer;