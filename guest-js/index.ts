import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

import type {
  MpvCommand,
  VideoMarginRatio,
  MpvConfig,
  MpvEvent,
  MpvCommandResponse,
  MpvPropertyEventFor,
} from './types';

export * from './types';

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

export const DEFAULT_MPV_CONFIG: MpvConfig = {
  mpvArgs: [
    '--no-config',
    '--vo=gpu-next',
    '--hwdec=auto-safe',
    '--media-controls=no',
  ],
  observedProperties: COMMON_PROPERTIES,
  ipcTimeoutMs: 2000,
  showMpvOutput: false,
};

/**
 * Initialize MPV player.
 * 
 * @example
 * ```typescript
 * import { destroyMpv, initializeMpv, MpvConfig } from 'tauri-plugin-mpv-api';
 * 
 * // Properties to observe
 * const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;
 * 
 * // MPV configuration
 * const mpvConfig: MpvConfig = {
 *   mpvArgs: [
 *     '--vo=gpu-next',
 *     '--hwdec=auto-safe',
 *   ],
 *   observedProperties: OBSERVED_PROPERTIES,
 * };
 * 
 * // Initialize MPV
 * try {
 *   await initializeMpv(mpvConfig);
 * } catch (error) {
 *   console.error('Failed to initialize MPV:', error);
 * }
 * 
 * // Destroy MPV when no longer needed
 * await destroyMpv();
 * ```
 *
 * @param {MpvConfig} [mpvConfig] - Initialization options.
 * @param {string} [windowLabel] - The label of the target window. Defaults to the current window's label.
 * @returns {Promise<string>} A promise that resolves with the actual window label used for initialization.
 *
 * @throws {Error} Throws an error if MPV initialization fails (e.g., mpv executable not in PATH).
 */
export async function initializeMpv(
  mpvConfig?: MpvConfig,
  windowLabel?: string,
): Promise<string> {

  mpvConfig = {
    ...DEFAULT_MPV_CONFIG,
    ...mpvConfig,
  };

  windowLabel = windowLabel ?? getCurrentWindow().label;

  return await invoke<string>('plugin:mpv|initialize_mpv', {
    mpvConfig,
    windowLabel,
  });
}

/**
 * Destroy MPV player.
 * 
 * @example
 * ```typescript
 * import { destroyMpv } from 'tauri-plugin-mpv-api';
 * 
 * await destroyMpv();
 * ```
 *
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<void>} A promise that resolves when the operation completes.
 */
export async function destroyMpv(windowLabel?: string): Promise<void> {
  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  return await invoke('plugin:mpv|destroy_mpv', {
    windowLabel,
  });
}

/**
 * Listen to MPV property change events.
 * 
 * @example
 * ```typescript
 * import { observeMpvProperties } from 'tauri-plugin-mpv-api';
 * 
 * const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;
 * 
 * // Observe properties
 * const unlisten = await observeMpvProperties(
 *   OBSERVED_PROPERTIES,
 *   ({ name, data }) => {
 *     switch (name) {
 *       case 'pause':
 *         console.log('Playback paused state:', data);
 *         break;
 *       case 'time-pos':
 *         console.log('Current time position:', data);
 *         break;
 *       case 'duration':
 *         console.log('Duration:', data);
 *         break;
 *       case 'filename':
 *         console.log('Current playing file:', data);
 *         break;
 *     }
 *   });
 * 
 * // Unlisten when no longer needed
 * unlisten();
 * ```
 *
 * @param {readonly string[]} properties - Properties to observe
 * @param {function} callback - Function to call when MPV events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 *
 */
export async function observeMpvProperties<T extends ReadonlyArray<string>>(
  properties: T,
  callback: (event: MpvPropertyEventFor<T[number]>) => void,
  windowLabel?: string
): Promise<UnlistenFn>;

/**
 * Listen to MPV property change events with common properties
 * 
 * @example
 * ```typescript
 * import { observeMpvProperties } from 'tauri-plugin-mpv-api';
 * 
 * // Observe properties
 * const unlisten = await observeMpvProperties(
 *   ({ name, data }) => {
 *     switch (name) {
 *       case 'pause':
 *         console.log('Playback paused state:', data);
 *         break;
 *       case 'time-pos':
 *         console.log('Current time position:', data);
 *         break;
 *       case 'duration':
 *         console.log('Duration:', data);
 *         break;
 *       case 'filename':
 *         console.log('Current playing file:', data);
 *         break;
 *     }
 *   });
 * 
 * // Unlisten when no longer needed
 * unlisten();
 * ```
 *
 * @param {function} callback - Function to call when MPV events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 *
 */
export async function observeMpvProperties(
  callback: (event: MpvPropertyEventFor<typeof COMMON_PROPERTIES[number]>) => void,
  windowLabel?: string
): Promise<UnlistenFn>;

export async function observeMpvProperties(
  arg1: ReadonlyArray<string> | ((event: any) => void),
  arg2?: ((event: any) => void) | string,
  arg3?: string
): Promise<UnlistenFn> {
  let properties: ReadonlyArray<string>;
  let callback: (event: any) => void;
  let windowLabel: string | undefined;

  if (typeof arg1 === 'function') {
    properties = COMMON_PROPERTIES;
    callback = arg1;
    windowLabel = arg2 as string | undefined;
  } else {
    properties = arg1;
    callback = arg2 as (event: any) => void;
    windowLabel = arg3;
  }

  return await listenMpvEvents(
    (mpvEvent) => {
      if (mpvEvent.event === 'property-change') {
        if (properties.includes(mpvEvent.name)) {
          callback(mpvEvent);
        }
      }
    },
    windowLabel,
  );
}

/**
 * Listen to all MPV events.
 * 
 * @example
 * ```typescript
 * import { listenMpvEvents } from 'tauri-plugin-mpv-api';
 * 
 * // Listen events
 * const unlisten = await listenMpvEvents((mpvEvent) => {
 *   if (mpvEvent.event === 'property-change') {
 *     const { name, data } = mpvEvent
 *     switch (name) {
 *       case 'pause':
 *         console.log('Playback paused state:', data);
 *         break;
 *       case 'time-pos':
 *         console.log('Current time position:', data);
 *         break;
 *       case 'duration':
 *         console.log('Duration:', data);
 *         break;
 *       case 'filename':
 *         console.log('Current playing file:', data);
 *         break;
 *     }
 *   }
 * });
 * 
 * // Unlisten when no longer needed
 * unlisten();
 * ```
 *
 * @param {(event: MpvEvent) => void} callback - Function to call when MPV events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 *
 */
export async function listenMpvEvents(
  callback: (event: MpvEvent) => void,
  windowLabel?: string
): Promise<UnlistenFn> {

  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  const eventName = `mpv-event-${windowLabel}`;

  return await listen<MpvEvent>(eventName, (event) => callback(event.payload));
}

/**
 * Send MPV command
 * 
 * @example
 * ```typescript
 * import { sendMpvCommand } from 'tauri-plugin-mpv-api';
 * 
 * // Load file
 * await sendMpvCommand({ command: ['loadfile', '/path/to/video.mp4'] });
 * 
 * // Play/pause
 * await sendMpvCommand({ command: ['set_property', 'pause', false] });
 * await sendMpvCommand({ command: ['set_property', 'pause', true] });
 * 
 * // Seek to position
 * await sendMpvCommand({ command: ['seek', 30, 'absolute'] });
 * await sendMpvCommand({ command: ['seek', 10, 'relative'] });
 * 
 * // Set volume
 * await sendMpvCommand({ command: ['set_property', 'volume', 80] });
 * 
 * // Get property
 * const duration = await sendMpvCommand({ command: ['get_property', 'duration'] });
 * console.log('Duration:', duration.data);
 * ```
 *
 * @param {MpvCommand} command - The command object to send to MPV. The `command` property is an array where the first element is the command name, followed by its parameters.
 * @param {string} [windowLabel] - Target window label, defaults to current window.
 * @returns {Promise<MpvCommandResponse>} A promise that resolves with the response from MPV.
 *
 * @throws {Error} Throws an error if the command fails or MPV returns an error status.
 *
 * @see {@link https://mpv.io/manual/master/#json-ipc} for a full list of commands.
 */
export async function sendMpvCommand(
  mpvCommand: MpvCommand,
  windowLabel?: string
): Promise<MpvCommandResponse> {

  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  return await invoke<MpvCommandResponse>('plugin:mpv|send_mpv_command', {
    mpvCommand,
    windowLabel,
  });
}

/**
 * Set video margin ratio
 * 
 * @example
 * ```typescript
 * import { setVideoMarginRatio } from 'tauri-plugin-mpv-api';
 * 
 * // Leave 10% space at bottom for control bar
 * await setVideoMarginRatio({ bottom: 0.1 });
 * 
 * // Leave margins on all sides
 * await setVideoMarginRatio({
 *   left: 0.05,
 *   right: 0.05,
 *   top: 0.1,
 *   bottom: 0.15
 * });
 * 
 * // Reset margins (remove all margins)
 * await setVideoMarginRatio({
 *   left: 0,
 *   right: 0,
 *   top: 0,
 *   bottom: 0
 * });
 * ```
 *
 * @param {VideoMarginRatio} ratio - Margin ratio configuration object
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<void>} Promise with no return value
 *
 * @throws {Error} Throws error when setting fails
 */
export async function setVideoMarginRatio(ratio: VideoMarginRatio, windowLabel?: string): Promise<void> {

  if (!windowLabel) {
    const currentWindow = getCurrentWindow();
    windowLabel = currentWindow.label;
  }

  return await invoke<void>('plugin:mpv|set_video_margin_ratio', {
    ratio,
    windowLabel,
  });
}