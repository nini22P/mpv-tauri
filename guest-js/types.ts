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
}

export interface MpvCommand {
  command: (string | boolean | number)[];
  request_id?: number;
}

export interface MpvCommandResponse {
  data?: any;
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

type SimpleEvent<E extends MpvEventType> = MpvEventBase<E>;

export interface PropertyChangeEvent<T extends keyof MpvPropertyTypes> extends MpvEventBase<'property-change'> {
  name: T;
  data: MpvPropertyTypes[T];
}

export interface StartFileEvent extends MpvEventBase<'start-file'> {
  playlist_entry_id?: number;
}

export interface EndFileEvent extends MpvEventBase<'end-file'> {
  reason: 'eof' | 'stop' | 'quit' | 'error' | 'redirect' | 'unknown';
  playlist_entry_id?: number;
  file_error?: string;
  playlist_insert_id?: number;
  playlist_insert_num_entries?: number;
}

export interface LogMessageEvent extends MpvEventBase<'log-message'> {
  prefix: string;
  level: string;
  text: string;
}

export interface HookEvent extends MpvEventBase<'hook'> {
  hook_id: number;
}

export interface CommandReplyEvent extends MpvEventBase<'get-property-reply' | 'set-property-reply' | 'command-reply'> {
  result?: any;
}

export interface ClientMessageEvent extends MpvEventBase<'client-message'> {
  args: string[];
}

export type MpvEvent<T extends string = string> =
  | SimpleEvent<'file-loaded' | 'seek' | 'playback-restart' | 'shutdown' | 'video-reconfig' | 'audio-reconfig'>
  | StartFileEvent
  | (T extends keyof MpvPropertyTypes ? PropertyChangeEvent<T> : never)
  | EndFileEvent
  | LogMessageEvent
  | HookEvent
  | CommandReplyEvent
  | ClientMessageEvent;

export type MpvEventListener<T extends string = typeof COMMON_PROPERTIES[number]> = (event: MpvEvent<T>) => void;

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