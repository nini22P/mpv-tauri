import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export * from './types';

import type {
  MpvCommand,
  VideoMarginRatio,
  MpvConfig,
  MpvEvent,
  MpvEventListener
} from './types';

import { COMMON_PROPERTIES } from './types';

/**
 * Initialize MPV player
 *
 * @description
 * Initialize MPV player instance, set properties to observe, configure MPV options, and return the window label.
 * If no properties list is specified, COMMON_PROPERTIES will be used as default.
 * If no window label is specified, the current window's label will be used automatically.
 * MPV configuration options can be provided to customize player behavior.
 *
 * @param {object} options - Initialization options object
 * @param {string[] | readonly string[]} [options.observedProperties] - Properties to observe
 * @param {string} [options.windowLabel] - Target window label
 * @param {MpvConfig} [options.mpvConfig] - MPV configuration options
 * @returns {Promise<string>} Returns the actual window label used
 *
 * @example
 * ```typescript
 *
 * const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;
 * 
 * await initializeMpv({
 *   observedProperties: OBSERVED_PROPERTIES,
 *   mpvConfig: {
 *     'vo': 'gpu',
 *     'hwdec': 'auto',
 *     'media-controls': 'no',
 *   }
 * });
 *
 * ```
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
    const currentWindow = getCurrentWindow();
    windowLabel = currentWindow.label;
  }

  return await invoke<string>('plugin:mpv|initialize_mpv', {
    observedProperties,
    windowLabel: windowLabel,
    mpvConfig: mpvConfig,
  });
}

/**
 * Listen to MPV events for the current window
 *
 * @description
 * Creates a window-specific event listener that only receives events from the MPV instance
 * running in the current window. This prevents cross-window event pollution.
 *
 * @template T - The type of the property names (should be from observedProperties)
 * @param {MpvEventListener<T>} callback - Function to call when MPV events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 *
 * @example
 * ```typescript
 * // Listen events
 * const unlisten = await listenMpvEvents<typeof OBSERVED_PROPERTIES[number]>((mpvEvent) => {
 *   if (mpvEvent.event === 'property-change') {
 *     console.log('Pause state changed:', mpvEvent.name, mpvEvent.data);
 *   }
 * });
 * ```
 */
export function listenMpvEvents<T extends string = typeof COMMON_PROPERTIES[number]>(
  callback: MpvEventListener<T>,
  windowLabel?: string
): Promise<UnlistenFn> {

  const targetWindow = windowLabel ?? getCurrentWindow().label;
  const eventName = `mpv-event-${targetWindow}`;

  console.log(`🎬 Setting up MPV event listener for window: ${targetWindow}, event: ${eventName}`);

  return listen<MpvEvent<T>>(eventName, (event) => callback(event.payload));
}

/**
 * Send MPV command
 *
 * @description
 * Send raw commands to MPV player. This is a low-level API that allows direct control of all MPV functionality.
 * Command format follows MPV's JSON IPC protocol.
 *
 * @param {MpvCommand} command - MPV command object
 * @param {(string|boolean|number)[]} command.command - Command array, first element is command name, followed by parameters
 * @returns {Promise<string>} MPV response result
 *
 * @example
 * ```typescript
 * // Play/pause
 * await sendMpvCommand({ command: ['set_property', 'pause', false] });
 * await sendMpvCommand({ command: ['set_property', 'pause', true] });
 *
 * // Load file
 * await sendMpvCommand({ command: ['loadfile', '/path/to/video.mp4'] });
 *
 * // Seek to position
 * await sendMpvCommand({ command: ['seek', 30, 'absolute'] });
 * await sendMpvCommand({ command: ['seek', 10, 'relative'] });
 *
 * // Set volume
 * await sendMpvCommand({ command: ['set_property', 'volume', 80] });
 *
 * // Get property
 * await sendMpvCommand({ command: ['get_property', 'duration'] });
 *
 * // Playlist operations
 * await sendMpvCommand({ command: ['playlist-next'] });
 * await sendMpvCommand({ command: ['playlist-prev'] });
 * ```
 *
 * @throws {Error} Throws error when command sending fails or MPV returns error
 *
 * @see {@link https://mpv.io/manual/master/#json-ipc} MPV JSON IPC Documentation
 */
export async function sendMpvCommand(command: MpvCommand, windowLabel?: string): Promise<string> {

  const commandJson = JSON.stringify(command);

  if (!windowLabel) {
    const currentWindow = getCurrentWindow();
    windowLabel = currentWindow.label;
  }

  return await invoke<string>('plugin:mpv|send_mpv_command', {
    commandJson,
    windowLabel,
  });
}

/**
 * Set video margin ratio
 *
 * @description
 * Set the margin ratio of video within the window, used to leave space around the video for UI controls.
 * Ratio values are decimals between 0-1, representing the proportion relative to window dimensions.
 *
 * @param {VideoMarginRatio} ratio - Margin ratio configuration object
 * @param {number} [ratio.left] - Left margin ratio (0-1)
 * @param {number} [ratio.right] - Right margin ratio (0-1)
 * @param {number} [ratio.top] - Top margin ratio (0-1)
 * @param {number} [ratio.bottom] - Bottom margin ratio (0-1)
 * @returns {Promise<void>} Promise with no return value
 *
 * @example
 * ```typescript
 * // Leave 10% space at bottom for control bar
 * await setVideoMarginRatio({ bottom: 0.1 });
 *
 * // Leave 5% space on left and right sides
 * await setVideoMarginRatio({ left: 0.05, right: 0.05 });
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