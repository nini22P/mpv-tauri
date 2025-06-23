import "./App.css";
import Control from "./components/Control";
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export interface MpvCommand {
  command: (string | boolean | number)[];
}

export type MpvEventType =
  | 'property-change'
  | 'file-loaded'
  | 'end-file';

export type MpvEventName =
  | 'filename'
  | 'pause'
  | 'eof-reached'
  | 'time-pos'
  | 'duration'
  | 'percent-pos';

export interface MpvEventPayload {
  event_type: MpvEventType;
  name?: MpvEventName;
  data?: any;
}

export interface PlayerStatus {
  isPaused: boolean;
  currentFile: string | null;
  eofReached: boolean;
  timePos: number;
  duration: number;
  percentPos: number;
}

export const sendCommand = async (command: MpvCommand) => {
  try {
    const commandJson = JSON.stringify(command);
    await invoke<string>('send_mpv_command', { commandJson });
  } catch (err) {
    console.error('Error sending MPV command:', err);
  }
};

function App() {
  const [status, setStatus] = useState<PlayerStatus>({
    isPaused: true,
    currentFile: null,
    eofReached: false,
    timePos: 0,
    duration: 0,
    percentPos: 0,
  });

  useEffect(() => {
    const unlistenPromise = listen<MpvEventPayload>('mpv-event', (event) => {
      const { event_type, name, data } = event.payload;

      setStatus(prev => {
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
      unlistenPromise.then(unlistenFn => unlistenFn());
    };
  }, []);

  useEffect(() => {
    getCurrentWindow().setTitle(status.currentFile ? `${status.currentFile} - MPV Tauri` : 'MPV Tauri');
  }, [status.currentFile])

  return (
    <main>
      <Control playerStatus={status} />
    </main>
  );
}

export default App;