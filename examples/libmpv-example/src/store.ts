
import { create, StoreApi, UseBoundStore } from 'zustand'

type WithSelectors<S> = S extends { getState: () => infer T }
  ? S & { use: { [K in keyof T]: () => T[K] } }
  : never

export const createSelectors = <S extends UseBoundStore<StoreApi<object>>>(
  _store: S,
) => {
  type State = ReturnType<S['getState']>

  const store = _store as WithSelectors<typeof _store>
  store.use = {}

  const keys = Object.keys(store.getState()) as Array<keyof State>

  for (const k of keys) {
    const selector = () => store(s => (s as State)[k]);
    (store.use as Record<keyof State, () => unknown>)[k] = selector
  }

  return store
}

export interface MpvPlaylistItem {
  filename: string;
  playing?: boolean;
  current?: boolean;
  title?: string;
  id: number;
  'playlist-path'?: string;
}

export interface PlayerStoreState {
  isInitalized: boolean;
  isPaused: boolean;
  playlist: MpvPlaylistItem[];
  filename: string | null;
  eofReached: boolean;
  timePos: number;
  duration: number;
  isFullscreen: boolean;
  volume: number | null;
  mute: boolean | null;
  speed: number | null;
  percentPos: number | null;
}

export interface PlayerStroeActions {
  updatePlayerState: <K extends keyof PlayerStoreState>(key: K, value: PlayerStoreState[K]) => void;
}

const usePlayerStoreBase = create<PlayerStoreState & PlayerStroeActions>((set) => ({
  integrationMode: 'wid',
  isInitalized: false,
  isPaused: true,
  filename: null,
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

const usePlayerStore = createSelectors(usePlayerStoreBase)

export default usePlayerStore
