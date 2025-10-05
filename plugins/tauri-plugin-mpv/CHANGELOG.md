## v0.5.0

- **BREAKING:** The `observeMpvProperties` function has been deprecated. Use `observeProperties` instead.
- **BREAKING:** The `listenMpvEvents` function has been deprecated. Use `listenEvents` instead.
- Add types for `start-file` and `end-file` events.
- Set `LC_NUMERIC` on setup.
- Allow overriding `wid` option.
- Re-license to MPL-2.0.

## v0.4.0

- The mpv instance is now automatically destroyed when its associated window is closed.
- The `MpvConfig` interface has been updated for clarity. `mpvPath` is now `path`, and `mpvArgs` is now `args`.

**Migration Example:**

```diff
const mpvConfig: MpvConfig = {
- mpvPath: 'path/to/mpv.exe',
+ path: 'path/to/mpv.exe',
- mpvArgs: [
+ args: [
    '--vo=gpu-next',
    '--hwdec=auto-safe',
    '--keep-open=yes',
    '--force-window',
  ],
  observedProperties: OBSERVED_PROPERTIES,
  ipcTimeoutMs: 2500,
}
```

## v0.3.2

- The `command` function now automatically generates a `request_id` if one is not provided.
- **BREAKING:** The core API functions have been renamed for brevity. Please update your code to use the new, shorter function names.

  - `initializeMpv` -> `init`
  - `sendMpvCommand` -> `command`
  - `destroyMpv` -> `destroy`

**Migration Example:**

```diff
- import { initializeMpv, destroyMpv, sendMpvCommand } from "tauri-plugin-mpv-api";
+ import { init, destroy, command } from "tauri-plugin-mpv-api";

- await initializeMpv(config);
+ await init(config);

- await sendMpvCommand({ command: ['loadfile', 'video.mp4'] });
+ await command('loadfile', ['video.mp4']); // Use the new shortcut for most commands
+ // The original object format is also available for advanced use cases (e.g., custom request_id)
+ // await command({ command: ['loadfile', 'video.mp4'] });

- await destroyMpv();
+ await destroy();
```

- Added getProperty and setProperty helper functions for easier property management.

**Usage Example:**

```ts
import { getProperty, setProperty } from "tauri-plugin-mpv-api";

// Set a property
await setProperty('volume', 80);

// Get a property
const volume = await getProperty('volume');
console.log('Current volume is:', volume);
```

## v0.2.7

- Integrate structured logging and improve error handling
- Improve events loop
- Use the built-in libmpv profile
- Replace global state with tauri managed state

## v0.2.0

- improve ipc path uniqueness and player state
- Propagate errors on mpv initialization failure
- Add observe properties function
- Improve destroy mpv

## v0.1.2

- Initial release
