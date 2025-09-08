import { useEffect } from "react";
import useAutoHide from "../hooks/useAutoHide";
import usePlayer from "../hooks/usePlayer";
import { getCurrentWindow } from "@tauri-apps/api/window";
import VideoRect from "./VideoRect";
import Control from "./Control";
import './Player.css';

const Player = ({ source }: { source: string | null }) => {
  const player = usePlayer();
  const { visible, show: showControls, hide: hideControls } = useAutoHide(5000);

  useEffect(() => {
    if (player.connection === 'connected') {
      showControls();
    }
  }, [player.connection]);

  useEffect(() => {
    if (player.connection === 'connected' && source) {
      player.loadFile(source);
    }
  }, [player.connection, source])

  useEffect(() => {
    getCurrentWindow().setTitle(player.currentFile ? `${player.currentFile} - mpv-tauri` : 'mpv-tauri');
  }, [player.currentFile])

  return (
    <div className="player" onMouseMove={showControls} onMouseLeave={hideControls}>
      <VideoRect connection={player.connection} />
      {
        player.isFullscreen
          ? visible && <div style={{ position: 'fixed', left: 0, right: 0, bottom: 0 }}>
            <Control player={player} />
          </div>
          : <Control player={player} />
      }
    </div>
  );
}

export default Player;