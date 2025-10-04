import {
  MpvConfig,
  init,
  observeProperties,
  command,
  setProperty,
  getProperty,
  destroy,
} from 'tauri-plugin-mpv-api'

// Properties to observe
const OBSERVED_PROPERTIES = ['pause', 'time-pos', 'duration', 'filename'] as const

// mpv configuration
const mpvConfig: MpvConfig = {
  args: [
    '--vo=gpu-next',
    '--hwdec=auto-safe',
    '--keep-open=yes',
    '--force-window',
  ],
  observedProperties: OBSERVED_PROPERTIES,
  ipcTimeoutMs: 2000,
}

try {
  await init(mpvConfig)
  console.log('mpv initialization completed successfully!')
} catch (error) {
  console.error('mpv initialization failed:', error)
}

// Observe properties
const unlisten = await observeProperties(
  OBSERVED_PROPERTIES,
  ({ name, data }) => {
    switch (name) {
      case 'pause':
        console.log('Playback paused state:', data)
        break
      case 'time-pos':
        console.log('Current time position:', data)
        break
      case 'duration':
        console.log('Duration:', data)
        break
      case 'filename':
        console.log('Current playing file:', data)
        break
    }
  })

// Unlisten when no longer needed
unlisten()

// Use the simple shortcut for most commands
await command('loadfile', ['/path/to/video.mp4'])
await command('seek', [10, 'relative']) // Seek 10 seconds forward

// Use the full object format if you need to provide a custom request_id
await command({ command: ['stop'], request_id: 123 })

// setProperty
await setProperty('volume', 75)

// getProperty
const volume = await getProperty('volume')
console.log('Current volume:', volume)

// Destroy mpv when your app closes or the player is no longer needed
await destroy()