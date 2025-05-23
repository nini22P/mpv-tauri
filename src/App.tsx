import "./App.css";
import Control from "./components/Control";
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from "@tauri-apps/api/core";

export interface MpvCommand {
  command: (string | boolean | number)[];
}

export interface MpvEventPayload {
  event_type: string;
  name?: string;
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
      // console.log('Received mpv event:', event.payload);
      const { event_type, name, data } = event.payload;

      setStatus(prev => {
        let newStatus = { ...prev };
        if (event_type === 'property-change') {
          if (name === 'pause') {
            newStatus.isPaused = data;
          } else if (name === 'filename') {
            newStatus.currentFile = data || null;
            if (data) {
              newStatus.isPaused = false;
              newStatus.eofReached = false;
              newStatus.timePos = 0;
              newStatus.duration = 0;
              newStatus.percentPos = 0;
            }
          } else if (name === 'eof-reached') {
            if (typeof data === 'boolean') {
              newStatus.eofReached = data;
              if (data) newStatus.isPaused = true;
            }
          } else if (name === 'time-pos') {
            newStatus.timePos = typeof data === 'number' ? data : newStatus.timePos;
          } else if (name === 'duration') {
            newStatus.duration = typeof data === 'number' ? data : newStatus.duration;
          } else if (name === 'percent-pos') {
            newStatus.percentPos = typeof data === 'number' ? data : newStatus.percentPos;
          }
        } else if (event_type === 'file-loaded') {
          sendCommand({ command: ['set_property', 'pause', false] });
          newStatus.eofReached = false;
        } else if (event_type === 'end-file') {
          newStatus.eofReached = true;
          if (data && data.reason === "quit") {
            newStatus.currentFile = null;
          }
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

  return (
    <main>
      <Control playerStatus={status} /> {/* */}
    </main>
  );
}

export default App;