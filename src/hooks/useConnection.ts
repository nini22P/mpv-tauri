import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import sendCommand from '../utils/sendCommand';

export type Connection = 'pending' | 'connected' | 'error';

interface MpvConnectionPayload {
  connected: boolean;
  error?: string;
}

const useConnection = () => {
  const [connection, setConnection] = useState<Connection>('pending');

  useEffect(() => {
    const unlistenConnection = listen<MpvConnectionPayload>('mpv-connection', async (event) => {
      console.log('mpv-connection', event.payload);
      if (event.payload.connected) {
        setConnection('connected');
        await sendCommand({ command: ['set_property', 'pause', true] });
      } else {
        setConnection('error');
        console.error('MPV connection failed:', event.payload.error);
      }
    });

    const connectionTimeout = setTimeout(() => {
      setConnection(currentConnection => {
        if (currentConnection === 'pending') {
          console.error('MPV connection timeout. Is mpv installed and in your PATH?');
          return 'error';
        }
        return currentConnection;
      });
    }, 10000);

    return () => {
      unlistenConnection.then(unlistenFn => unlistenFn());
      clearTimeout(connectionTimeout);
    };
  }, [])

  return connection;
};

export default useConnection;
