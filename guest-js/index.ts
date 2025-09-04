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

import { COMMON_PROPERTIES } from './types';

export * from './types';

/**
 * Initialize MPV player
 * 
 * @example
 * ```typescript
 * import { initializeMpv } from 'tauri-plugin-mpv-api';
 * 
 * const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;
 * 
 * try {
 *   await initializeMpv({
 *     observedProperties: Array.from(OBSERVED_PROPERTIES),
 *     mpvConfig: {
 *       'vo': 'gpu-next',
 *       'hwdec': 'auto',
 *       'media-controls': 'no',
 *     }
 *   });
 * } catch (error) {
 *   console.error('Failed to initialize MPV:', error);
 * }
 * ```
 *
 * @param {object} options - Initialization options object
 * @param {string[] | readonly string[]} [options.observedProperties] - Properties to observe
 * @param {string} [options.windowLabel] - Target window label
 * @param {MpvConfig} [options.mpvConfig] - MPV configuration options
 * @returns {Promise<string>} Returns the actual window label used
 *
 * @throws {Error} Throws error when MPV initialization fails
 */
export async function initializeMpv(
  {
    observedProperties,
    windowLabel,
    mpvConfig,
  }: {
    observedProperties?: string[] | readonly string[],
    windowLabel?: string,
    mpvConfig?: MpvConfig,
  } = {}
): Promise<string> {

  if (!observedProperties) {
    observedProperties = Array.from(COMMON_PROPERTIES);
  }

  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  return await invoke<string>('plugin:mpv|initialize_mpv', {
    observedProperties,
    windowLabel: windowLabel,
    mpvConfig: mpvConfig,
  });
}

/**
 * Listen to MPV property change events for the current window
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
 * @param {string[]} properties - Properties to observe
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
 * Listen to MPV property change events for the current window with common properties
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

  const unlisten = await listenMpvEvents((event) => {
    if (event.event === 'property-change') {
      if (properties.includes(event.name)) {
        callback(event);
      }
    }
  }, windowLabel);

  return unlisten;
}

/**
 * Listen to all MPV events for the current window
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
 * @template T - The type of the property names (should be from observedProperties)
 * @param {MpvEventListener<T>} callback - Function to call when MPV events are received
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

  try {
    const unlisten = await listen<MpvEvent>(eventName, (event) => callback(event.payload));
    console.log(`✅ Raw MPV event listener is active for window: ${windowLabel}`);
    return unlisten;
  } catch (error) {
    console.error(`❌ Failed to set up raw MPV event listener for window: ${windowLabel}`, error);
    return Promise.reject(error);
  }
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
 * @param {MpvCommand} command - MPV command object
 * @param {(string|boolean|number)[]} command.command - Command array, first element is command name, followed by parameters
 * @returns {Promise<MpvCommandResponse>} MPV response result
 *
 * @throws {Error} Throws error when command sending fails or MPV returns error
 *
 * @see {@link https://mpv.io/manual/master/#json-ipc}
 */
export async function sendMpvCommand(
  command: MpvCommand,
  windowLabel?: string
): Promise<MpvCommandResponse> {

  const commandJson = JSON.stringify(command);

  if (!windowLabel) {
    windowLabel = getCurrentWindow().label;
  }

  const response = await invoke<string>('plugin:mpv|send_mpv_command', {
    commandJson,
    windowLabel,
  });

  return JSON.parse(response);
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
 * @param {number} [ratio.left] - Left margin ratio (0-1)
 * @param {number} [ratio.right] - Right margin ratio (0-1)
 * @param {number} [ratio.top] - Top margin ratio (0-1)
 * @param {number} [ratio.bottom] - Bottom margin ratio (0-1)
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