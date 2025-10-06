export type MpvFormat = 'string' | 'flag' | 'int64' | 'double' | 'node';

export type MpvFormatToType = {
  string: string;
  flag: boolean;
  int64: number;
  double: number;
  node: unknown;
};

export type MpvObservableFormat = MpvFormat;

export type MpvObservableProperty =
  | readonly [string, MpvObservableFormat]
  | readonly [string, MpvObservableFormat, 'none', ...unknown[]];

export type MpvPropertyData<T extends MpvObservableProperty> =
  T extends readonly [
    infer _,
    infer TFormat extends MpvObservableFormat,
    'none',
    ...unknown[],
  ]
  ? MpvFormatToType[TFormat] | null
  : T extends readonly [infer _, infer TFormat extends MpvObservableFormat]
  ? MpvFormatToType[TFormat]
  : never;

export interface MpvConfig {
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

export interface MpvLogMessageEvent extends MpvEventBase<'log-message'> {
  prefix: string;
  level: string;
  text: string;
}

export interface MpvGetPropertyReplyEvent extends MpvEventBase<'get-property-reply'> {
  name: string;
  data: string | boolean | number | unknown | null;
  error: number;
  id: number;
}

export interface MpvSetPropertyReplyEvent extends MpvEventBase<'set-property-reply'> {
  error: number;
  id: number;
}

export interface MpvCommandReplyEvent extends MpvEventBase<'command-reply'> {
  result: string | boolean | number | unknown | null;
  error: number;
  id: number;
}

export interface MpvStartFileEvent extends MpvEventBase<'start-file'> {
  playlist_entry_id: number;
}

export interface MpvEndFileEvent extends MpvEventBase<'end-file'> {
  reason: 'eof' | 'stop' | 'quit' | 'error' | 'redirect' | 'unknown';
  error: number;
  playlist_entry_id: number;
  playlist_insert_id: number;
  playlist_insert_num_entries: number;
}

export type MpvFileLoadedEvent = MpvEventBase<'file-loaded'>

export type MpvIdleEvent = MpvEventBase<'idle'>

export type MpvTickEvent = MpvEventBase<'tick'>

export interface MpvClientMessageEvent extends MpvEventBase<'client-message'> {
  args: string[];
}

export type MpvVideoReconfigEvent = MpvEventBase<'video-reconfig'>
export type MpvAudioReconfigEvent = MpvEventBase<'audio-reconfig'>
export type MpvSeekEvent = MpvEventBase<'seek'>
export type MpvPlaybackRestartEvent = MpvEventBase<'playback-restart'>

export interface MpvPropertyChangeEvent<TName extends string, TValue>
  extends MpvEventBase<'property-change'> {
  name: TName;
  data: TValue;
  id: number;
}

export type MpvEventFromProperties<T extends MpvObservableProperty> = T extends readonly [
  infer TName extends string,
  ...unknown[],
]
  ? MpvPropertyChangeEvent<TName, MpvPropertyData<T>>
  : never;

export type MpvQueueOverflowEvent = MpvEventBase<'queue-overflow'>

export interface MpvHookEvent extends MpvEventBase<'hook'> {
  hook_id: number;
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