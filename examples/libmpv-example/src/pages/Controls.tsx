import { open } from '@tauri-apps/plugin-dialog'
import './Controls.css'
import formatTime from '../utils/formatTime'
import { useState } from 'react'
import usePlayerStore from '../store'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { loadFile, seek, stop, play, pause, playlistPrev, playlistNext, playlistPlay } from '../utils/commands'

const Controls = () => {

  const isPaused = usePlayerStore.use.isPaused()
  const playlist = usePlayerStore.use.playlist()
  const timePos = usePlayerStore.use.timePos()
  const duration = usePlayerStore.use.duration()
  const filename = usePlayerStore.use.filename()
  const eofReached = usePlayerStore.use.eofReached()
  const isFullscreen = usePlayerStore.use.isFullscreen()
  const updatePlayerState = usePlayerStore.use.updatePlayerState()

  const [playlistVisible, setPlaylistVisible] = useState(false)

  const handleLoadFile = async (folder = false) => {
    const file = await open({
      multiple: false,
      directory: folder,
    })

    if (file) {
      await loadFile(file)
    }
  }

  const toggleFullscreen = async () => {
    await getCurrentWindow().setFullscreen(!isFullscreen)
    updatePlayerState('isFullscreen', !isFullscreen)
  }

  return (
    <div className="control">
      <div className="control-buttons">
        <button
          type="button"
          title={playlistVisible ? 'Hide Playlist' : 'Playlist'}
          style={{ fontSize: '1.125rem' }}
          onClick={async () => {
            setPlaylistVisible(!playlistVisible)
          }}
        >
          {playlistVisible ? '×' : '≡'}
        </button>
        <button type="button" title={'Load File'} onClick={() => handleLoadFile()} >📄</button>
        <button type="button" title={'Load Folder'} onClick={() => handleLoadFile(true)} >📂</button>
        <button
          type="button"
          title={isPaused ? 'Play' : 'Pause'}
          onClick={isPaused
            ? async () => {
              if (filename && (duration - timePos < 0.5 || eofReached)) {
                await seek(0)
              };
              play()
            }
            : pause}
        >
          {isPaused ? '▶' : '⏸'}
        </button>
        <button
          type="button"
          title={'Stop'}
          onClick={
            () => {
              stop()
              updatePlayerState('timePos', 0)
              updatePlayerState('duration', 0)
              updatePlayerState('filename', null)
            }
          }
        >⏹</button>
        <button type="button" title={'Previous'} onClick={playlistPrev} >⏮</button>
        <button type="button" title={'Next'} onClick={playlistNext} >⏭</button>
        <button
          type="button"
          title={isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
          onClick={toggleFullscreen} >
          {isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
        </button>
      </div>
      <input
        className="slider"
        title='Slider'
        type='range'
        min={0}
        max={duration * 1000}
        value={timePos * 1000}
        step={1}
        onChange={(e) => seek(Number(e.target.value) / 1000)}
      />
      <p className="time"> {formatTime(timePos)} / {formatTime(duration)}</p>

      {
        playlistVisible &&
        <div className="playlist">
          {
            playlist.map((item, index) => (
              <div
                key={index}
                className={`playlist-item ${item.current ? 'active' : ''}`}
                onClick={() => {
                  playlistPlay(index)
                  setPlaylistVisible(false)
                }}
              >
                {item.current ? '▶ ' : ''}{item.filename.split('/').pop()?.split('\\').pop()}
              </div>
            ))
          }
        </div>
      }
    </div>
  )
}

export default Controls