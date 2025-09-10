import { MpvPlaylistItem } from 'tauri-plugin-mpv-api';
import { create, StoreApi, UseBoundStore } from 'zustand'

type WithSelectors<S> = S extends { getState: () => infer T }
  ? S & { use: { [K in keyof T]: () => T[K] } }
  : never

const createSelectors = <S extends UseBoundStore<StoreApi<object>>>(
  _store: S,
) => {
  const store = _store as WithSelectors<typeof _store>
  store.use = {}
  for (const k of Object.keys(store.getState())) {
    ; (store.use as any)[k] = () => store((s) => s[k as keyof typeof s])
  }

  return store
}

export type Connection = 'pending' | 'connected' | 'error';

export interface PlayerStoreState {
  connection: Connection;
  isPaused: boolean;
  playlist: MpvPlaylistItem[];
  currentFile: string | undefined;
  eofReached: boolean;
  timePos: number;
  duration: number;
  isFullscreen: boolean;
  volume: number | undefined;
  mute: boolean | undefined;
  speed: number | undefined;
  percentPos: number | undefined;
}

export interface PlayerStroeActions {
  setConnection: (connection: PlayerStoreState['connection']) => void;
  setIsPaused: (isPaused: PlayerStoreState['isPaused']) => void;
  setCurrentFile: (currentFile: PlayerStoreState['currentFile']) => void;
  setPlaylist: (playlist: PlayerStoreState['playlist']) => void;
  setEofReached: (eofReached: PlayerStoreState['eofReached']) => void;
  setTimePos: (timePos: PlayerStoreState['timePos']) => void;
  setDuration: (duration: PlayerStoreState['duration']) => void;
  setIsFullscreen: (isFullscreen: PlayerStoreState['isFullscreen']) => void;
  setVolume: (volume: PlayerStoreState['volume']) => void;
  setMute: (mute: PlayerStoreState['mute']) => void;
  setSpeed: (speed: PlayerStoreState['speed']) => void;
  setPercentPos: (percentPos: PlayerStoreState['percentPos']) => void;
}

const usePlayerStoreBase = create<PlayerStoreState & PlayerStroeActions>((set) => ({
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
  setConnection: (connection) => set({ connection }),
  setIsPaused: (isPaused) => set({ isPaused }),
  setCurrentFile: (currentFile) => set({ currentFile }),
  setPlaylist: (playlist) => set({ playlist }),
  setEofReached: (eofReached) => set({ eofReached }),
  setTimePos: (timePos) => set({ timePos }),
  setDuration: (duration) => set({ duration }),
  setIsFullscreen: (isFullscreen) => set({ isFullscreen }),
  setVolume: (volume) => set({ volume }),
  setMute: (mute) => set({ mute }),
  setSpeed: (speed) => set({ speed }),
  setPercentPos: (percentPos) => set({ percentPos }),
}))

const usePlayerStore = createSelectors(usePlayerStoreBase);

export default usePlayerStore;
