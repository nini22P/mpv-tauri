export type MpvFormat = 'string' | 'flag' | 'int64' | 'double' | 'node';

export type MpvFormatToType = {
  string: string;
  flag: boolean;
  int64: number;
  double: number;
  node: unknown;
};

export type MpvObservableFormat = Exclude<MpvFormat, 'node'>;

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
  | 'client-message'
  | 'video-reconfig'
  | 'audio-reconfig'
  | 'seek'
  | 'playback-restart'
  | 'property-change'
  | 'queue-overflow'
  | 'deprecated';

interface MpvEventBase<E extends MpvEventType> {
  event: E;
}

export type MpvShutdownEvent = MpvEventBase<'shutdown'>

export interface MpvLogMessageEvent extends MpvEventBase<'log-message'> {
  prefix: string;
  level: string;
  text: string;
  log_level: 'debug' | 'error' | 'fatal' | 'info' | 'none' | 'warn' | 'v' | 'trace';
}

export interface MpvGetPropertyReplyEvent extends MpvEventBase<'get-property-reply'> {
  name: string;
  result: string | boolean | number | unknown;
  reply_userdata: number;
}

export interface MpvSetPropertyReplyEvent extends MpvEventBase<'set-property-reply'> {
  reply_userdata: number;
}

export interface MpvCommandReplyEvent extends MpvEventBase<'command-reply'> {
  reply_userdata: number;
}

export type MpvStartFileEvent = MpvEventBase<'start-file'>

export interface MpvEndFileEvent extends MpvEventBase<'end-file'> {
  reason: 'eof' | 'stop' | 'quit' | 'error' | 'redirect';
}

export type MpvFileLoadedEvent = MpvEventBase<'file-loaded'>

export interface MpvClientMessageEvent extends MpvEventBase<'client-message'> {
  message: string[];
}

export type MpvVideoReconfigEvent = MpvEventBase<'video-reconfig'>
export type MpvAudioReconfigEvent = MpvEventBase<'audio-reconfig'>
export type MpvSeekEvent = MpvEventBase<'seek'>
export type MpvPlaybackRestartEvent = MpvEventBase<'playback-restart'>

export interface MpvPropertyChangeEvent<TName extends string, TValue>
  extends MpvEventBase<'property-change'> {
  name: TName;
  change: TValue;
  reply_userdata: number;
}

export type MpvEventFromProperties<T extends MpvObservableProperty> = T extends readonly [
  infer TName extends string,
  infer TFormat extends MpvObservableFormat
]
  ? MpvPropertyChangeEvent<TName, MpvFormatToType[TFormat]>
  : never;

export type MpvQueueOverflowEvent = MpvEventBase<'queue-overflow'>
export type MpvDeprecatedEvent = MpvEventBase<'deprecated'>

export type MpvEvent =
  | MpvShutdownEvent
  | MpvLogMessageEvent
  | MpvGetPropertyReplyEvent
  | MpvSetPropertyReplyEvent
  | MpvCommandReplyEvent
  | MpvStartFileEvent
  | MpvEndFileEvent
  | MpvFileLoadedEvent
  | MpvClientMessageEvent
  | MpvVideoReconfigEvent
  | MpvAudioReconfigEvent
  | MpvSeekEvent
  | MpvPlaybackRestartEvent
  | MpvEventFromProperties<MpvObservableProperty>
  | MpvQueueOverflowEvent
  | MpvDeprecatedEvent;

export interface VideoMarginRatio {
  left?: number;
  right?: number;
  top?: number;
  bottom?: number;
}