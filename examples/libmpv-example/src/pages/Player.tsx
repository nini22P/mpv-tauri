import { useEffect } from 'react'
import useAutoHide from '../hooks/useAutoHide'
import { getCurrentWindow } from '@tauri-apps/api/window'
import VideoRect from './VideoRect'
import Controls from './Controls'
import './Player.css'
import usePlayerStore from '../store'
import useMpv from '../hooks/useMpv'
import { loadFile } from '../utils/commands'

const Player = ({ source }: { source: string | null }) => {
  useMpv()

  const { visible, show: showControls, hide: hideControls } = useAutoHide(5000)

  const isInitalized = usePlayerStore.use.isInitalized()
  const filename = usePlayerStore.use.filename()
  const isFullscreen = usePlayerStore.use.isFullscreen()

  useEffect(() => {
    if (isInitalized && source) {
      loadFile(source)
    }
  }, [isInitalized, source])

  useEffect(() => {
    getCurrentWindow().setTitle(filename ? `${filename} - tauri-plugin-libmpv-example` : 'tauri-plugin-libmpv-example')
  }, [filename])

  return (
    <div
      className="player"
      onMouseMove={showControls}
      onMouseLeave={hideControls}
    >
      <VideoRect isInitalized={isInitalized} />
      {
        isFullscreen
          ?
          visible &&
          <div style={{ position: 'fixed', left: 0, right: 0, bottom: 0 }}>
            <Controls />
          </div>
          : <Controls />
      }
    </div>
  )
}

export default Player