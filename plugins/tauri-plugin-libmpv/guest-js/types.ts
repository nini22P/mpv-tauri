export type MpvFormat = 'string' | 'flag' | 'int64' | 'double' | 'node';

export type MpvFormatToType = {
  string: string;
  flag: boolean;
  int64: number;
  double: number;
  node: unknown;
};

export type MpvObservableFormat = MpvFormat;

export type MpvObservableProperty = readonly [string, MpvObservableFormat];

export interface MpvConfig {
  integrationMode?: 'wid' | 'render';
  initialOptions?: Record<string, string | boolean | number>;
  observedProperties?: readonly MpvObservableProperty[];
}

export type MpvEventType =
  | 'shutdown'
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

export type MpvShutdownEvent = MpvEventBase<'shutdown'>

export interface LogMessage {
  prefix: string;
  level: string;
  text: string;
  log_level: 'none' | 'fatal' | 'error' | 'warn' | 'info' | 'v' | 'debug' | 'trace' | 'unknown';
}

export interface MpvLogMessageEvent extends MpvEventBase<'log-message'> {
  data: LogMessage;
}

export interface MpvGetPropertyReplyEvent extends MpvEventBase<'get-property-reply'> {
  name: string;
  data: string | boolean | number | unknown;
  error: number;
  reply_userdata: number;
}

export interface MpvSetPropertyReplyEvent extends MpvEventBase<'set-property-reply'> {
  error: number;
  reply_userdata: number;
}

export interface MpvCommandReplyEvent extends MpvEventBase<'command-reply'> {
  data: string | boolean | number | unknown;
  error: number;
  reply_userdata: number;
}

export interface StartFile {
  playlist_entry_id: number;
}

export interface MpvStartFileEvent extends MpvEventBase<'start-file'> {
  data: StartFile;
}

export type EndFileReason = 'eof' | 'stop' | 'quit' | 'error' | 'redirect' | 'unknown';

export type EndFile = {
  reason: EndFileReason;
  error: number;
  playlist_entry_id: number;
  playlist_insert_id: number;
  playlist_insert_num_entries: number;
};

export interface MpvEndFileEvent extends MpvEventBase<'end-file'> {
  data: EndFile;
}

export type MpvFileLoadedEvent = MpvEventBase<'file-loaded'>

export type MpvIdleEvent = MpvEventBase<'idle'>

export type MpvTickEvent = MpvEventBase<'tick'>

export interface MpvClientMessageEvent extends MpvEventBase<'client-message'> {
  data: string[];
}

export type MpvVideoReconfigEvent = MpvEventBase<'video-reconfig'>
export type MpvAudioReconfigEvent = MpvEventBase<'audio-reconfig'>
export type MpvSeekEvent = MpvEventBase<'seek'>
export type MpvPlaybackRestartEvent = MpvEventBase<'playback-restart'>

export interface MpvPropertyChangeEvent<TName extends string, TValue>
  extends MpvEventBase<'property-change'> {
  name: TName;
  data: TValue;
  reply_userdata: number;
}

export type MpvEventFromProperties<T extends MpvObservableProperty> = T extends readonly [
  infer TName extends string,
  infer TFormat extends MpvObservableFormat
]
  ? MpvPropertyChangeEvent<TName, MpvFormatToType[TFormat]>
  : never;

export type MpvQueueOverflowEvent = MpvEventBase<'queue-overflow'>

export interface Hook {
  id: number;
}

export interface MpvHookEvent extends MpvEventBase<'hook'> {
  data: Hook;
}

export type MpvEvent =
  | MpvShutdownEvent
  | MpvLogMessageEvent
  | MpvGetPropertyReplyEvent
  | MpvSetPropertyReplyEvent
  | MpvCommandReplyEvent
  | MpvStartFileEvent
  | MpvEndFileEvent
  | MpvFileLoadedEvent
  | MpvIdleEvent
  | MpvTickEvent
  | MpvClientMessageEvent
  | MpvVideoReconfigEvent
  | MpvAudioReconfigEvent
  | MpvSeekEvent
  | MpvPlaybackRestartEvent
  | MpvEventFromProperties<MpvObservableProperty>
  | MpvQueueOverflowEvent
  | MpvHookEvent;

export interface VideoMarginRatio {
  left?: number;
  right?: number;
  top?: number;
  bottom?: number;
}