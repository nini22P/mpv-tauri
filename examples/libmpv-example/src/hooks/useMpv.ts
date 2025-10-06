import { useEffect } from 'react'
import { init, observeProperties, MpvConfig, destroy, MpvObservableProperty, listenEvents } from 'tauri-plugin-libmpv-api'
import usePlayerStore, { MpvPlaylistItem } from '../store'

const OBSERVED_PROPERTIES = [
  ['playlist', 'node'],
  ['filename', 'string', 'none'],
  ['pause', 'flag'],
  ['eof-reached', 'flag', 'none'],
  ['time-pos', 'double', 'none'],
  ['duration', 'double', 'none'],
  ['volume', 'double'],
  ['mute', 'flag'],
  ['speed', 'double'],
] as const satisfies MpvObservableProperty[]

const useMpv = () => {

  const updatePlayerState = usePlayerStore.use.updatePlayerState()

  useEffect(() => {
    (async () => {
      await destroy()
      updatePlayerState('isInitalized', false)

      const mpvConfig: MpvConfig = {
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
  }, [])

  useEffect(() => {
    const unlistenPromise = listenEvents(
      (mpvEvent) => {
        if (mpvEvent.event == 'property-change' && mpvEvent.name !== 'time-pos') {
          console.log(mpvEvent)
        } else if (mpvEvent.event !== 'property-change') {
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
      ({ name, data }) => {
        switch (name) {
          case 'playlist':
            updatePlayerState('playlist', data as MpvPlaylistItem[])
            break
          case 'filename':
            updatePlayerState('filename', data)
            break
          case 'pause':
            updatePlayerState('isPaused', data)
            break
          case 'eof-reached':
            updatePlayerState('eofReached', data ?? false)
            break
          case 'time-pos':
            updatePlayerState('timePos', data ?? 0)
            break
          case 'duration':
            updatePlayerState('duration', data ?? 0)
            break
          case 'volume':
            updatePlayerState('volume', data)
            break
          case 'mute':
            updatePlayerState('mute', data)
            break
          case 'speed':
            updatePlayerState('speed', data)
            break
          default:
            console.log(name, data)
            break
        }
      })

    return () => {
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [OBSERVED_PROPERTIES])
}

export default useMpv