import { open } from '@tauri-apps/plugin-dialog';
import './Control.css';
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

  return (
    <div className="control">
      <div className="control-buttons">
        <button
          type="button"
          title={playlistVisible ? 'Hide Playlist' : 'Playlist'}
          style={{ fontSize: '1.125rem' }}
          onClick={() => setPlaylistVisible(!playlistVisible)}
        >
          {playlistVisible ? '×' : '≡'}
        </button>
        <button type="button" title={'Load File'} onClick={() => handleLoadFile()} >📄</button>
        <button type="button" title={'Load Folder'} onClick={() => handleLoadFile(true)} >📂</button>
        <button
          type="button"
          title={player.isPaused ? 'Play' : 'Pause'}
          onClick={player.isPaused ? player.play : player.pause}
        >
          {player.isPaused ? '▶' : '⏸'}
        </button>
        <button type="button" title={'Stop'} onClick={player.stop} >⏹</button>
        <button type="button" title={'Previous'} onClick={player.playlistPrev} >⏮</button>
        <button type="button" title={'Next'} onClick={player.playlistNext} >⏭</button>
        <button
          type="button"
          title={player.isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
          onClick={player.toggleFullscreen} >
          {player.isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
        </button>
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