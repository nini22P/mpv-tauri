import { MpvPropertyTypes } from "./generated/mpv-properties";

export interface MpvConfig {
  initialProperties?: Record<string, string | boolean | number>;
  observedProperties?: readonly string[];
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