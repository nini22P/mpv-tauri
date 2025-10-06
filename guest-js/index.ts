import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

import type {
  MpvCommand,
  VideoMarginRatio,
  MpvConfig,
  MpvEvent,
  MpvCommandResponse,
  MpvPropertyEventFor,
  MpvPropertyValue,
} from './types'

export * from './types'

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
] as const

export const DEFAULT_MPV_CONFIG: MpvConfig = {
  args: [
    '--vo=gpu-next',
    '--hwdec=auto-safe',
    '--keep-open=yes',
    '--force-window',
  ],
  observedProperties: COMMON_PROPERTIES,
  ipcTimeoutMs: 2000,
  showMpvOutput: false,
}


/**
 * Initialize mpv player.
 * 
 * @param {MpvConfig} [mpvConfig] - Initialization options.
 * @param {string} [windowLabel] - The label of the target window. Defaults to the current window's label.
 * @returns {Promise<string>} A promise that resolves with the actual window label used for initialization.
 * @throws {Error} Throws an error if mpv initialization fails (e.g., mpv executable not in PATH).
 * 
 * @example
 * ```typescript
 * import { init, destroy, MpvConfig } from 'tauri-plugin-mpv-api';
 * 
 * // Properties to observe
 * const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;
 * 
 * // mpv configuration
 * const mpvConfig: MpvConfig = {
 *   args: [
 *     '--vo=gpu-next',
 *     '--hwdec=auto-safe',
 *     '--keep-open=yes',
 *     '--force-window',
 *   ],
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
 */
export async function init(
  mpvConfig?: MpvConfig,
  windowLabel?: string,
): Promise<string> {

  mpvConfig = {
    ...DEFAULT_MPV_CONFIG,
    ...mpvConfig,
  }

  windowLabel = windowLabel ?? getCurrentWindow().label

  return await invoke<string>('plugin:mpv|init', {
    mpvConfig,
    windowLabel,
  })
}


/**
 * Destroy mpv player.
 * 
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<void>} A promise that resolves when the operation completes.
 * 
 * @example
 * ```typescript
 * import { destroy } from 'tauri-plugin-mpv-api';
 * 
 * await destroy();
 * ```
 */
export async function destroy(windowLabel?: string): Promise<void> {

  windowLabel = windowLabel ?? getCurrentWindow().label

  return await invoke('plugin:mpv|destroy', {
    windowLabel,
  })
}


/**
 * Listen to mpv property change events.
 * 
 * @param {readonly string[]} properties - Properties to observe
 * @param {function} callback - Function to call when mpv events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 * 
 * @example
 * ```typescript
 * import { observeProperties } from 'tauri-plugin-mpv-api';
 * 
 * const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const;
 * 
 * // Observe properties
 * const unlisten = await observeProperties(
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
 */
export async function observeProperties<T extends ReadonlyArray<string>>(
  properties: T,
  callback: (event: MpvPropertyEventFor<T[number]>) => void,
  windowLabel?: string
): Promise<UnlistenFn>;

/**
 * Listen to mpv property change events with common properties
 * 
 * @param {function} callback - Function to call when mpv events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 * 
 * @example
 * ```typescript
 * import { observeProperties } from 'tauri-plugin-mpv-api';
 * 
 * // Observe properties
 * const unlisten = await observeProperties(
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
 */
export async function observeProperties(
  callback: (event: MpvPropertyEventFor<typeof COMMON_PROPERTIES[number]>) => void,
  windowLabel?: string
): Promise<UnlistenFn>;

export async function observeProperties(
  arg1: ReadonlyArray<string> | ((event: never) => void),
  arg2?: ((event: never) => void) | string,
  arg3?: string
): Promise<UnlistenFn> {
  let properties: ReadonlyArray<string>
  let callback: (event: unknown) => void
  let windowLabel: string | undefined

  if (typeof arg1 === 'function') {
    properties = COMMON_PROPERTIES
    callback = arg1 as (event: unknown) => void
    windowLabel = arg2 as string | undefined
  } else {
    properties = arg1
    callback = arg2 as (event: unknown) => void
    windowLabel = arg3
  }

  return await listenEvents(
    (mpvEvent) => {
      if (mpvEvent.event === 'property-change') {
        if (properties.includes(mpvEvent.name)) {
          callback(mpvEvent)
        }
      }
    },
    windowLabel,
  )
}


/**
 * Listen to all mpv events.
 * 
 * @param {(event: MpvEvent) => void} callback - Function to call when mpv events are received
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<UnlistenFn>} Function to call to stop listening
 * 
 * @example
 * ```typescript
 * import { listenEvents } from 'tauri-plugin-mpv-api';
 * 
 * // Listen events
 * const unlisten = await listenEvents((mpvEvent) => {
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
 */
export async function listenEvents(
  callback: (event: MpvEvent) => void,
  windowLabel?: string
): Promise<UnlistenFn> {

  windowLabel = windowLabel ?? getCurrentWindow().label

  const eventName = `mpv-event-${windowLabel}`

  return await listen<MpvEvent>(eventName, (event) => callback(event.payload))
}


/**
 * Sends a command to mpv and returns only the `data` portion of the response.
 * This is a convenient shortcut for commands where you only need the return value.
 *
 * @param name The name of the command to execute.
 * @param args (Optional) An array of arguments for the command.
 * @param windowLabel (Optional) The label of the Tauri window to target. Defaults to the current window.
 * @returns A promise that resolves with the data returned by mpv.
 * @throws {Error} Throws an error if the mpv command fails.
 *
 * @example
 * ```typescript
 * import { command } from 'tauri-plugin-mpv-api';
 *
 * // Get the duration property
 * const duration = await command('get_property', ['duration']);
 * console.log('Duration is:', duration);
 *
 * // Seek 10 seconds forward (args are optional)
 * await command('seek', [10, 'relative']);
 *
 * // Pause the player (no args needed)
 * await command('cycle', ['pause']);
 * ```
 */
export async function command(
  name: string,
  args?: unknown[],
  windowLabel?: string
): Promise<unknown>;

/**
 * Sends a command to mpv without arguments and returns the `data` portion of the response.
 *
 * @param name The name of the command to execute.
 * @param windowLabel (Optional) The label of the Tauri window to target. Defaults to the current window.
 * @returns A promise that resolves with the data returned by mpv.
 * @throws {Error} Throws an error if the mpv command fails.
 */
export async function command(
  name: string,
  windowLabel?: string
): Promise<unknown>;

/**
 * Sends a command to mpv using original JSON IPC object structure.
 *
 * @param mpvCommand The command object to send to mpv.
 * @param windowLabel (Optional) The label of the Tauri window to target. Defaults to the current window.
 * @returns A promise that resolves with the full response object from mpv.
 * @throws {Error} Throws an error if the command fails or mpv returns an error status.
 * @see {@link https://mpv.io/manual/master/#json-ipc} for a full list of commands.
 *
 * @example
 * ```typescript
 * import { command } from 'tauri-plugin-mpv-api';
 *
 * // Load a file and check the full response
 * const response = await command({ command: ['loadfile', '/path/to/video.mp4'] });
 * if (response.error === 'success') {
 * console.log('File loaded successfully');
 * }
 *
 * // Seek with a custom request_id for tracking
 * const seekResponse = await command({ command: ['seek', 10, 'relative'], request_id: 999 });
 * console.log('Seek command with ID', seekResponse.request_id, 'was successful.');
 * ```
 */
export async function command(
  mpvCommand: MpvCommand,
  windowLabel?: string
): Promise<MpvCommandResponse>;

export async function command(
  arg1: MpvCommand | string,
  arg2?: unknown[] | string,
  arg3?: string
): Promise<MpvCommandResponse | unknown> {
  let finalMpvCommand: MpvCommand
  let finalWindowLabel: string | undefined
  let isShortcut = false

  if (typeof arg1 === 'string') {
    isShortcut = true
    const name = arg1
    const args = Array.isArray(arg2) ? arg2 : []
    finalWindowLabel = Array.isArray(arg2) ? arg3 : arg2

    finalMpvCommand = {
      command: [name, ...args],
    }
  } else {
    isShortcut = false
    finalMpvCommand = arg1
    finalWindowLabel = arg2 as string
  }

  finalWindowLabel = finalWindowLabel ?? getCurrentWindow().label

  const response = await invoke<MpvCommandResponse>('plugin:mpv|command', {
    mpvCommand: finalMpvCommand,
    windowLabel: finalWindowLabel,
  })

  if (isShortcut) {
    if (response.error !== 'success') {
      throw new Error(`mpv command failed: ${response.error}`)
    }
    return response.data
  } else {
    return response
  }
}


/**
 * Gets the value of an mpv property.
 *
 * @param name The name of the property to get.
 * @param windowLabel (Optional) The label of the Tauri window to target.
 * @returns A promise that resolves with the typed property value.
 * @throws {Error} Throws an error if the command fails.
 *
 * @example
 * ```typescript
 * import { getProperty } from 'tauri-plugin-mpv-api';
 *
 * // Get volume
 * const volume = await getProperty('volume');
 *
 * // `custom` is now treated as `string`
 * const custom = await getProperty<string>('my-custom-property');
 * ```
 */
export async function getProperty<
  T = never,
  K extends string = string
>(
  name: K,
  windowLabel?: string
): Promise<[T] extends [never] ? MpvPropertyValue<K> : T> {
  const value = await command('get_property', [name], windowLabel)
  return value as [T] extends [never] ? MpvPropertyValue<K> : T
}


/**
 * Sets the value of an mpv property.
 *
 * @param name The name of the property to set.
 * @param value The value to set. Must match the property's type if it is known.
 * @param windowLabel (Optional) The label of the Tauri window to target.
 * @returns A promise that resolves when the property has been set.
 * @throws {Error} Throws an error if the command fails.
 *
 * @example
 * ```typescript
 * import { setProperty } from 'tauri-plugin-mpv-api';
 *
 * // Set volume
 * await setProperty('volume', 80);
 *
 * // Explicitly provide a type for a custom or unknown property
 * await setProperty<string>('my-custom-property', 'some-value');
 * ```
 */
export async function setProperty<
  T = never,
  K extends string = string
>(
  name: K,
  value: [T] extends [never] ? MpvPropertyValue<K> : T,
  windowLabel?: string
): Promise<void> {
  await command('set_property', [name, value], windowLabel)
}


/**
 * Set video margin ratio
 * 
 * @param {VideoMarginRatio} ratio - Margin ratio configuration object
 * @param {string} [windowLabel] - Target window label, defaults to current window
 * @returns {Promise<void>} Promise with no return value
 * @throws {Error} Throws error when setting fails
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
 *   bottom: 0.15,
 * });
 * 
 * // Reset margins (remove all margins)
 * await setVideoMarginRatio({
 *   left: 0,
 *   right: 0,
 *   top: 0,
 *   bottom: 0,
 * });
 * ```
 */
export async function setVideoMarginRatio(ratio: VideoMarginRatio, windowLabel?: string): Promise<void> {

  windowLabel = windowLabel ?? getCurrentWindow().label

  return await invoke<void>('plugin:mpv|set_video_margin_ratio', {
    ratio,
    windowLabel,
  })
}

/**
 * @deprecated Use `init()` instead. This function will be removed in a future version.
 */
export const initializeMpv = init

/**
 * @deprecated Use `destroy()` instead. This function will be removed in a future version.
 */
export const destroyMpv = destroy

/**
 * @deprecated Use `observeProperties()` instead. This function will be removed in a future version.
 */
export const observeMpvProperties = observeProperties

/**
 * @deprecated Use `listenEvents()` instead. This function will be removed in a future version.
 */
export const listenMpvEvents = listenEvents

/**
 * @deprecated Use `command()` instead. This function will be removed in a future version.
 */
export const sendMpvCommand = command