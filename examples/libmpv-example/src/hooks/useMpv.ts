import { useEffect } from 'react'
import { init, observeProperties, MpvConfig, destroy, MpvObservableProperty, listenEvents } from 'tauri-plugin-libmpv-api'
import usePlayerStore from '../store'

const OBSERVED_PROPERTIES = [
  ['playlist', 'string'],
  ['filename', 'string'],
  ['pause', 'flag'],
  ['eof-reached', 'flag'],
  ['time-pos', 'double'],
  ['duration', 'double'],
  ['volume', 'double'],
  ['mute', 'flag'],
  ['speed', 'double'],
] as const satisfies MpvObservableProperty[]

const useMpv = () => {

  const integrationMode = usePlayerStore.use.integrationMode()
  const updatePlayerState = usePlayerStore.use.updatePlayerState()

  useEffect(() => {
    (async () => {
      const mpvConfig: MpvConfig = {
        integrationMode,
        initialOptions: {
          'vo': 'gpu-next',
          'hwdec': 'auto-safe',
          'keep-open': 'yes',
          'force-window': 'yes',
          'pause': 'yes',
        },
        observedProperties: OBSERVED_PROPERTIES,
      }

      try {
        console.log('Initializing mpv with properties:', OBSERVED_PROPERTIES)
        await init(mpvConfig)
        console.log('mpv initialization completed successfully!')
        updatePlayerState('isInitalized', true)
      } catch (error) {
        console.error('mpv initialization failed:', error)
      }

    })()

    return () => {
      (async () => {
        await destroy()
        updatePlayerState('isInitalized', false)
      })()
    }
  }, [integrationMode])

  useEffect(() => {
    const handleBeforeUnload = (_event: BeforeUnloadEvent) => destroy()
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload)
    }
  }, [])

  useEffect(() => {
    const unlistenPromise = listenEvents(
      (mpvEvent) => {
        if (mpvEvent.event !== 'property-change') {
          console.log(mpvEvent)
        }
      })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [])

  useEffect(() => {
    const unlistenPromise = observeProperties(
      OBSERVED_PROPERTIES,
      ({ name, change }) => {
        if (name !== 'time-pos')
          console.log(name, change)
        switch (name) {
          case 'playlist':
            updatePlayerState('playlist', JSON.parse(change))
            break
          case 'filename':
            updatePlayerState('filename', change)
            break
          case 'pause':
            updatePlayerState('isPaused', change)
            break
          case 'eof-reached':
            updatePlayerState('eofReached', change ?? false)
            break
          case 'time-pos':
            updatePlayerState('timePos', change ?? 0)
            break
          case 'duration':
            updatePlayerState('duration', change ?? 0)
            break
          case 'volume':
            updatePlayerState('volume', change)
            break
          case 'mute':
            updatePlayerState('mute', change)
            break
          case 'speed':
            updatePlayerState('speed', change)
            break
          default:
            console.log(name, change)
            break
        }
      })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [OBSERVED_PROPERTIES])
}

export default useMpv