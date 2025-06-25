import { open } from '@tauri-apps/plugin-dialog';
import './Control.css';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Player } from '../hooks/usePlayer';
import formatTime from '../utils/formatTime';
import { useState } from 'react';

const Control = ({ player }: { player: Player }) => {

  const [playlistVisible, setPlaylistVisible] = useState(false);

  const handleLoadFile = async (folder = false) => {
    const file = await open({
      multiple: false,
      directory: folder,
    });

    if (file) {
      await player.loadFile(file);
    }
  };

  const handleToggleFullscreen = async () => await getCurrentWindow().setFullscreen(!await getCurrentWindow().isFullscreen());

  return (
    <div className="control">
      <div className="control-buttons">
        <button type="button" onClick={() => setPlaylistVisible(!playlistVisible)} >Playlist</button>
        <button type="button" onClick={() => handleLoadFile()} >Load File</button>
        <button type="button" onClick={() => handleLoadFile(true)} >Load Folder</button>
        <button
          type="button"
          onClick={player.isPaused ? player.play : player.pause}
        >
          {player.isPaused ? 'Play' : 'Pause'}
        </button>
        <button type="button" onClick={player.stop} >Stop</button>
        <button type="button" onClick={player.seekBackward} >-10s</button>
        <button type="button" onClick={player.seekForward} >+10s</button>
        <button type="button" onClick={handleToggleFullscreen} >Toggle Fullscreen</button>
      </div>
      <input
        className="slider"
        title='Slider'
        type='range'
        min={0}
        max={player.duration}
        value={player.timePos}
        step={1}
        onChange={(e) => player.seek(Number(e.target.value))}
      />
      <p className="time"> {formatTime(player.timePos)} / {formatTime(player.duration)}</p>

      {
        playlistVisible &&
        <div className="playlist">
          {
            player.playlist.map((item, index) => (
              <div
                key={index}
                className={`playlist-item ${item.current ? 'active' : ''}`}
                onClick={() => {
                  player.playlistPlay(index);
                  setPlaylistVisible(false);
                }}
              >
                {item.current ? '▶ ' : ''}{item.filename.split('/').pop()?.split('\\').pop()}
              </div>
            ))
          }
        </div>
      }
    </div>
  );
};

export default Control;