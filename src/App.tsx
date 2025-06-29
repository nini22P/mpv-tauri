import { useEffect, useState } from "react";
import "./App.css";
import Control from "./components/Control";
import useConnection from "./hooks/useConnection";
import usePlayer from "./hooks/usePlayer";
import useAutoHide from "./hooks/useAutoHide";
import { getCurrentWindow } from "@tauri-apps/api/window";
import useCli from "./hooks/useCli";
import VideoRect from "./components/VideoRect";
import useFullscreen from "./hooks/useFullscreen";

function App() {

  const connection = useConnection();
  const player = usePlayer();
  const source = useCli();
  const { isFullscreen, toggleFullscreen } = useFullscreen();
  const { visible, show: showControls, hide: hideControls } = useAutoHide(5000);

  useEffect(() => {
    if (connection === 'connected') {
      showControls();
    }
  }, [connection, showControls]);

  useEffect(() => {
    if (connection === 'connected' && source) {
      player.loadFile(source);
    }
  }, [connection, source])

  useEffect(() => {
    getCurrentWindow().setTitle(player.currentFile ? `${player.currentFile} - MPV Tauri` : 'MPV Tauri');
  }, [player.currentFile])

  return (
    <main
      className="app"
      onMouseMove={showControls}
      onMouseLeave={hideControls}
    >
      <VideoRect />
      {
        isFullscreen
          ? visible && <div style={{ position: 'fixed', left: 0, right: 0, bottom: 0 }}>
            <Control player={player} isFullscreen={isFullscreen} toggleFullscreen={toggleFullscreen} />
          </div>
          : <Control player={player} isFullscreen={isFullscreen} toggleFullscreen={toggleFullscreen} />
      }
    </main>
  );
}

export default App;