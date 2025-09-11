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
  filename: string | undefined;
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
  updatePlayerState: <K extends keyof PlayerStoreState>(key: K, value: PlayerStoreState[K]) => void;
}

const usePlayerStoreBase = create<PlayerStoreState & PlayerStroeActions>((set) => ({
  connection: 'pending',
  isPaused: true,
  filename: undefined,
  playlist: [],
  eofReached: false,
  timePos: 0,
  duration: 0,
  isFullscreen: false,
  volume: 100,
  mute: false,
  speed: 1.0,
  percentPos: 0,
  updatePlayerState: (key, value) => set({ [key]: value }),
}))

const usePlayerStore = createSelectors(usePlayerStoreBase);

export default usePlayerStore;
