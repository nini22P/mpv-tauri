import { useEffect } from "react";
import "./App.css";
import Control from "./components/Control";
import usePlayer from "./hooks/usePlayer";
import useAutoHide from "./hooks/useAutoHide";
import { getCurrentWindow } from "@tauri-apps/api/window";
import useCli from "./hooks/useCli";
import VideoRect from "./components/VideoRect";

function App() {

  const player = usePlayer();
  const source = useCli();
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
    getCurrentWindow().setTitle(player.currentFile ? `${player.currentFile} - MPV Tauri` : 'MPV Tauri');
  }, [player.currentFile])

  return (
    <main
      className="app"
      onMouseMove={showControls}
      onMouseLeave={hideControls}
    >
      <VideoRect connection={player.connection} />
      {
        player.isFullscreen
          ? visible && <div style={{ position: 'fixed', left: 0, right: 0, bottom: 0 }}>
            <Control player={player} />
          </div>
          : <Control player={player} />
      }
    </main>
  );
}

export default App;