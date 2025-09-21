export interface MpvConfig {
  mpvPath?: string;
  mpvArgs?: string[];
  observedProperties?: readonly string[];
  ipcTimeoutMs?: number;
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
  command: (string | boolean | number)[];

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

export interface VideoMarginRatio {
  left?: number;
  right?: number;
  top?: number;
  bottom?: number;
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