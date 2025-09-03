/**
 * MPV command interface
 * @interface MpvCommand
 */
export interface MpvCommand {
  command: (string | boolean | number)[];
}

/**
 * Video margin ratio configuration
 * @interface VideoMarginRatio
 */
export interface VideoMarginRatio {
  /** Left margin ratio (0-1) */
  left?: number;
  /** Right margin ratio (0-1) */
  right?: number;
  /** Top margin ratio (0-1) */
  top?: number;
  /** Bottom margin ratio (0-1) */
  bottom?: number;
}

/**
 * MPV configuration options
 * @interface MpvConfig
 */
export interface MpvConfig {
  /** MPV configuration options as key-value pairs */
  [key: string]: string | number | boolean;
}

/**
 * MPV event
 *
 * @see {@link https://mpv.io/manual/master/#events} MPV Events Documentation
 */
export type MpvEventName =
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

/**
 * Base interface for all MPV events
 * @template E - The event name type
 * @interface MpvEventBase
 */
interface MpvEventBase<E extends MpvEventName> {
  /** Event name */
  event: E;
  /** Error string if an error occurred */
  error?: string;
  /** Reply user data ID */
  id?: number;
}

/**
 * Simple event type for events without additional data
 * @template E - The event name type
 */
type SimpleEvent<E extends MpvEventName> = MpvEventBase<E>;

/**
 * Property change event interface
 * @template T - The property name type
 * @template D - The property data type
 * @interface PropertyChangeEvent
 */
export interface PropertyChangeEvent<T extends string = string, D = unknown> extends MpvEventBase<'property-change'> {
  /** Property name that changed */
  name: T;
  /** New value of the property */
  data: D;
}

/**
 * Start file event interface
 * @interface StartFileEvent
 */
export interface StartFileEvent extends MpvEventBase<'start-file'> {
  /** Playlist entry ID of the file being loaded */
  playlist_entry_id?: number;
}

/**
 * End file event interface
 * @interface EndFileEvent
 */
export interface EndFileEvent extends MpvEventBase<'end-file'> {
  /** Reason why playback ended */
  reason: 'eof' | 'stop' | 'quit' | 'error' | 'redirect' | 'unknown';
  /** Playlist entry ID of the file that was being played */
  playlist_entry_id?: number;
  /** Error string describing why playback failed */
  file_error?: string;
  /** Playlist entry ID of the first inserted entry (for redirects) */
  playlist_insert_id?: number;
  /** Total number of inserted playlist entries */
  playlist_insert_num_entries?: number;
}

/**
 * Log message event interface
 * @interface LogMessageEvent
 */
export interface LogMessageEvent extends MpvEventBase<'log-message'> {
  /** Module prefix identifying the sender */
  prefix: string;
  /** Log level */
  level: string;
  /** Log message text */
  text: string;
}

/**
 * Hook event interface
 * @interface HookEvent
 */
export interface HookEvent extends MpvEventBase<'hook'> {
  /** ID to pass to mpv_hook_continue() */
  hook_id: number;
}

/**
 * Command reply event interface
 * @interface CommandReplyEvent
 */
export interface CommandReplyEvent extends MpvEventBase<'get-property-reply' | 'set-property-reply' | 'command-reply'> {
  /** Result of the operation */
  result?: any;
}

/**
 * Client message event interface
 * @interface ClientMessageEvent
 */
export interface ClientMessageEvent extends MpvEventBase<'client-message'> {
  /** Array of strings with message data */
  args: string[];
}

/**
 * Union type of all possible MPV events
 * @template T - The property name type (should be from observedProperties)
 */
export type MpvEvent<T extends string = string> =
  | SimpleEvent<'file-loaded' | 'seek' | 'playback-restart' | 'shutdown' | 'video-reconfig' | 'audio-reconfig'>
  | StartFileEvent
  | PropertyChangeEvent<T>
  | EndFileEvent
  | LogMessageEvent
  | HookEvent
  | CommandReplyEvent
  | ClientMessageEvent;

/**
 * MPV event listener callback
 * @template T - The type of the property name (should be from observedProperties)
 */
export type MpvEventListener<T extends string = typeof COMMON_PROPERTIES[number]> = (event: MpvEvent<T>) => void;

/**
 * Common MPV properties list
 *
 * @description
 * Contains properties for playback control, audio control, and basic status information.
 * Suitable for most player applications, providing basic playback functionality monitoring.
 *
 * @constant
 * @readonly
 */
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
  'percent-pos'    // Playback progress percentage
] as const;