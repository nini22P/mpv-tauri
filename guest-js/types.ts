/**
 * @see {@link https://mpv.io/manual/master/#command-interface-playlist}
 */
export interface MpvPlaylistItem {
  filename: string;
  playing?: boolean;
  current?: boolean;
  title?: string;
  id: number;
  'playlist-path'?: string;
}

/**
 * @see {@link https://mpv.io/manual/master/#properties}
 */
export interface MpvPropertyTypes {
  'playlist': MpvPlaylistItem[];
  'filename'?: string;
  'pause': boolean;
  'eof-reached'?: boolean;
  'time-pos'?: number;
  'duration'?: number;
  'volume': number;
  'mute': boolean;
  'speed': number;
  'percent-pos'?: number;
}

export interface MpvCommand {
  command: (string | boolean | number)[];
  request_id?: number;
}

export interface MpvCommandResponse {
  data: unknown | null;
  error: 'success' | string;
  request_id: number;
}

export interface VideoMarginRatio {
  left?: number;
  right?: number;
  top?: number;
  bottom?: number;
}

export interface MpvConfig {
  [key: string]: string | number | boolean;
}

/**
 * @see {@link https://mpv.io/manual/master/#list-of-events}
 */
export type MpvEventType =
  | 'start-file'
  | 'end-file'
  | 'file-loaded'
  | 'seek'
  | 'playback-restart'
  | 'shutdown'
  | 'log-message'
  | 'hook'
  | 'get-property-reply'
  | 'set-property-reply'
  | 'command-reply'
  | 'client-message'
  | 'video-reconfig'
  | 'audio-reconfig'
  | 'property-change'

interface MpvEventBase<E extends MpvEventType> {
  event: E;
  error?: string;
  id?: number;
}

export interface PropertyChangeEvent extends MpvEventBase<'property-change'> {
  name: string;
  data: unknown;
}

export interface OtherMpvEvent extends MpvEventBase<Exclude<MpvEventType, 'property-change'>> {
  [key: string]: unknown;
}

export type MpvEvent =
  | PropertyChangeEvent
  | OtherMpvEvent;

export type MpvPropertyEventFor<K extends string> = {
  [P in K]: P extends keyof MpvPropertyTypes
  ? { name: P; data: MpvPropertyTypes[P] }
  : { name: P; data: unknown };
}[K];

export const COMMON_PROPERTIES = [
  'playlist',      // Playlist
  'filename',      // Current filename
  'pause',         // Pause state
  'eof-reached',   // End of file reached state
  'time-pos',      // Playback position (seconds)
  'duration',      // Total duration (seconds)
  'volume',        // Volume (0-100)
  'mute',          // Mute state
  'speed',         // Playback speed
] as const;