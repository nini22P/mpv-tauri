export interface MpvConfig {
  /** Path to the mpv executable. If not provided, it will be searched in the system's PATH. */
  path?: string;
  /** An array of command-line arguments to pass to the mpv instance. */
  args?: string[];
  /** A list of mpv properties to observe automatically upon initialization. */
  observedProperties?: readonly string[];
  /** Timeout in milliseconds for IPC commands. */
  ipcTimeoutMs?: number;
  /** Whether to show mpv's console output in the terminal. */
  showMpvOutput?: boolean;
}

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
  'playback-time'?: number;
  'playtime-remaining'?: number;
}

/**
 * Represents a command to be sent to the mpv player instance.
 */
export interface MpvCommand {
  /**
   * An array containing the command name and its arguments.
   * For a list of available commands, see the mpv command interface documentation.
   * @example
   * // Loads a video file
   * ['loadfile', 'path/to/video.mp4']
   * // Seeks 10 seconds forward
   * ['seek', 10, 'relative']
   */
  command: unknown[];

  /**
   * A unique identifier for the command.
   * If not provided, the plugin will generate a unique ID automatically.
   * This ID is used to match the command with its corresponding response.
   * @see MpvCommandResponse.request_id
   */
  request_id?: number;
}

/**
 * Represents the response received from mpv after executing a command.
 */
export interface MpvCommandResponse {
  /**
   * The data returned by the command on success.
   * The format of the data depends on the command executed.
   * For commands that do not return a value, this property may be null or undefined.
   * @example
   * // The result of a 'get_property' command for 'volume' might be `100`.
   * data: 100
   */
  data?: unknown;

  /**
   * Indicates the result of the command execution.
   * It will be the string 'success' if the command completed without errors.
   * Otherwise, it will contain a string describing the error.
   */
  error: 'success' | string;

  /**
   * The unique identifier that matches the ID sent in the original `MpvCommand`.
   * This allows you to correlate responses with the commands you sent.
   * @see MpvCommand.request_id
   */
  request_id: number;
}

/**
 * @see {@link https://mpv.io/manual/master/#list-of-events}
 */
export type MpvEventType =
  | 'log-message'
  | 'get-property-reply'
  | 'set-property-reply'
  | 'command-reply'
  | 'start-file'
  | 'end-file'
  | 'file-loaded'
  | 'idle'
  | 'tick'
  | 'client-message'
  | 'video-reconfig'
  | 'audio-reconfig'
  | 'seek'
  | 'playback-restart'
  | 'property-change'
  | 'queue-overflow'
  | 'hook';

interface MpvEventBase<E extends MpvEventType> {
  event: E;
}

export interface MpvStartFileEvent extends MpvEventBase<'start-file'> {
  playlist_entry_id: number;
}

export type EndFileReason = 'eof' | 'stop' | 'quit' | 'error' | 'redirect' | 'unknown';

export interface MpvEndFileEvent extends MpvEventBase<'end-file'> {
  reason: EndFileReason;
  playlist_entry_id: number;
}

export type MpvPropertyEventFor<K extends string> = {
  [P in K]: P extends keyof MpvPropertyTypes
  ? { name: P; data: MpvPropertyTypes[P]; id: number }
  : { name: P; data?: unknown; id: number };
}[K];

export type MpvPropertyValue<K extends string> = K extends keyof MpvPropertyTypes
  ? MpvPropertyTypes[K]
  : unknown;

export interface PropertyChangeEvent extends MpvEventBase<'property-change'> {
  name: string;
  data?: unknown;
  id: number;
}

export interface OtherMpvEvent extends MpvEventBase<Exclude<MpvEventType, 'property-change'>> {
  [key: string]: unknown;
}

export type MpvEvent =
  | MpvStartFileEvent
  | MpvEndFileEvent
  | PropertyChangeEvent
  | OtherMpvEvent;

export interface VideoMarginRatio {
  left?: number;
  right?: number;
  top?: number;
  bottom?: number;
}