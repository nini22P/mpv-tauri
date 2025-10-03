import { useEffect } from 'react'
import { observeProperties, MpvConfig, destroy, init, listenEvents } from 'tauri-plugin-mpv-api'
import usePlayerStore from '../store'

const OBSERVED_PROPERTIES = [
  'playlist',
  'filename',
  'pause',
  'eof-reached',
  'time-pos',
  'duration',
  'volume',
  'mute',
  'speed',
] as const

const useMpv = () => {

  const connection = usePlayerStore.use.connection()

  const updatePlayerState = usePlayerStore.use.updatePlayerState()

  useEffect(() => {
    (async () => {
      const mpvConfig: MpvConfig = {
        args: [
          '--vo=gpu-next',
          '--hwdec=auto-safe',
          '--keep-open=yes',
          '--force-window',
          '--pause',
        ],
        observedProperties: OBSERVED_PROPERTIES,
        ipcTimeoutMs: 2500,
      }

      try {
        await init(mpvConfig)
        console.log('mpv initialization completed successfully!')
        updatePlayerState('connection', 'connected')
      } catch (error) {
        console.error('mpv initialization failed:', error)
        updatePlayerState('connection', 'error')
      }
    })()
  }, [])

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
        if (connection !== 'connected')
          updatePlayerState('connection', 'connected')
        switch (name) {
          case 'playlist':
            updatePlayerState('playlist', data)
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