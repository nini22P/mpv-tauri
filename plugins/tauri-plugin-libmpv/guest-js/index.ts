import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

import type {
  VideoMarginRatio,
  MpvConfig,
  MpvEvent,
  MpvPropertyEventFor,
} from './types';

export * from './types';
export * from './generated/mpv-properties'

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
  initialProperties: {
    "vo": "gpu-next",
    "hwdec": "auto-safe",
    "keep-open": "yes",
    "force-window": "yes",
    "pause": "yes",
  },
  observedProperties: COMMON_PROPERTIES,
};

/**
 * Initialize mpv player.
 * 
 * @example
 * ```typescript
 * import { destroy, init, MpvConfig } from 'tauri-plugin-libmpv-api';
 * 
 * // Properties to observe
 * const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;
 * 
 * // mpv configuration
 * const mpvConfig: MpvConfig = {
 *   initialProperties: {
 *     "vo": "gpu-next",
 *     "hwdec": "auto-safe",
 *     "keep-open": "yes",
 *     "force-window": "yes",
 *     "pause": "yes",
 *   },
 *   observedProperties: OBSERVED_PROPERTIES,
 * };
 * 
 * // Initialize mpv
 * try {
 *   await init(mpvConfig);
 * } catch (error) {
 *   console.error('Failed to initialize mpv:', error);
 * }
 * 
 * // Destroy mpv when no longer needed
 * await destroy();
 * ```
 *
 * @param {MpvConfig} [mpvConfig] - Initialization options.
 * @param {string} [windowLabel] - The label of the target window. Defaults to the current window's label.
 * @returns {Promise<string>} A promise that resolves with the actual window label used for initialization.
 *
 * @throws {Error} Throws an error if mpv initialization fails (e.g., mpv executable not in PATH).
 */
export async function init(
  mpvConfig?: MpvConfig,
  windowLabel?: string,
): Promise<string> {

  mpvConfig = {
    ...DEFAULT_MPV_CONFIG,
    ...mpvConfig,
  };

  windowLabel = windowLabel ?? getCurrentWindow().label;

  return await invoke<string>('plugin:libmpv|init', {
    mpvConfig,
    windowLabel,
  });
}

/**
 * Destroy mpv player.
 * 
 * @example
 * ```typescript
 * import { destroy } from 'tauri-plugin-libmpv-api';
 * 
 * await destroy();
 * ```
 *
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<void>} A promise that resolves when the operation completes.
 */
export async function destroy(windowLabel?: string): Promise<void> {
  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  return await invoke('plugin:libmpv|destroy', {
    windowLabel,
  });
}

/**
 * Listen to mpv property change events.
 * 
 * @example
 * ```typescript
 * import { observeMpvProperties } from 'tauri-plugin-libmpv-api';
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
 * @param {function} callback - Function to call when mpv events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 *
 */
export async function observeProperties<T extends ReadonlyArray<string>>(
  properties: T,
  callback: (event: MpvPropertyEventFor<T[number]>) => void,
  windowLabel?: string
): Promise<UnlistenFn>;

/**
 * Listen to mpv property change events with common properties
 * 
 * @example
 * ```typescript
 * import { observeMpvProperties } from 'tauri-plugin-libmpv-api';
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
 * @param {function} callback - Function to call when mpv events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 * 
 * @see {@link https://mpv.io/manual/master/#properties} for a full list of properties.
 *
 */
export async function observeProperties(
  callback: (event: MpvPropertyEventFor<typeof COMMON_PROPERTIES[number]>) => void,
  windowLabel?: string
): Promise<UnlistenFn>;

export async function observeProperties(
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

  return await listenEvents(
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
 * Listen to all mpv events.
 * 
 * @example
 * ```typescript
 * import { listenMpvEvents } from 'tauri-plugin-libmpv-api';
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
 * @param {(event: MpvEvent) => void} callback - Function to call when mpv events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 *
 */
export async function listenEvents(
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
 * Send mpv command
 * 
 * @example
 * ```typescript
 * import { command } from 'tauri-plugin-libmpv-api';
 * 
 * // Load file
 * await command('loadfile', ['/path/to/video.mp4']);
 * 
 * // Play/pause
 * await command('set', ['pause', false]);
 * await command('set', ['pause', true]);
 * 
 * // Seek to position
 * await command('seek', [30, 'absolute']);
 * await command('seek', [10, 'relative']);
 * 
 * // Set volume
 * await command('set', ['volume', 80]);
 * 
 * ```
 *
 * @param {string} name - Command name
 * @param {Array<string | boolean | number>} [args] - Command arguments
 * @param {string} [windowLabel] - Target window label, defaults to current window.
 *
 * @throws {Error} Throws an error if the command fails or mpv returns an error status.
 *
 */
export async function command(
  name: string,
  args: (string | boolean | number)[] = [],
  windowLabel?: string
) {

  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  await invoke('plugin:libmpv|command', {
    name,
    args,
    windowLabel,
  });
}

/**
 * Set mpv property
 * 
 * @example
 * ```typescript
 * import { setProperty } from 'tauri-plugin-libmpv-api';
 * 
 * // Play/pause
 * await setProperty('pause', false);
 * await setProperty('pause', true);
 * 
 * // Set volume
 * await setProperty('volume', 80);
 * 
 * ```
 *
 * @param {string} name - Property name
 * @param {string | boolean | number} value - Property value
 * @param {string} [windowLabel] - Target window label, defaults to current window.
 *
 * @throws {Error} Throws an error if the command fails or mpv returns an error status.
 *
 * @see {@link https://mpv.io/manual/master/#properties} for a full list of properties.
 */
export async function setProperty(
  name: string,
  value: string | boolean | number,
  windowLabel?: string,
) {

  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  await invoke('plugin:libmpv|set_property', {
    name,
    value,
    windowLabel,
  });
}

/**
 * Get mpv property
 * 
 * @example
 * ```typescript
 * import { getProperty } from 'tauri-plugin-libmpv-api';
 * 
 * // Get volume
 * const volume = await getProperty('volume');
 * console.log('Volume:', volume);
 * 
 * ```
 *
 * @param {string} name - Property name
 * @param {string} [windowLabel] - Target window label, defaults to current window.
 * @returns {Promise<unknown>} Promise with property value
 *
 * @throws {Error} Throws an error if the command fails or mpv returns an error status.
 *
 * @see {@link https://mpv.io/manual/master/#properties} for a full list of properties.
 */

export async function getProperty<T extends string>(
  name: T,
  windowLabel?: string,
) {

  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  return await invoke<unknown>('plugin:libmpv|get_property', {
    name,
    windowLabel,
  });
}

/**
 * Set video margin ratio
 * 
 * @example
 * ```typescript
 * import { setVideoMarginRatio } from 'tauri-plugin-libmpv-api';
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

  return await invoke<void>('plugin:libmpv|set_video_margin_ratio', {
    ratio,
    windowLabel,
  });
}