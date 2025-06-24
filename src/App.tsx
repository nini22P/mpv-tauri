import { useEffect } from "react";
import "./App.css";
import Control from "./components/Control";
import useConnection from "./hooks/useConnection";
import usePlayer from "./hooks/usePlayer";
import useAutoHide from "./hooks/useAutoHide";
import { getCurrentWindow } from "@tauri-apps/api/window";
import useCli from "./hooks/useCli";

function App() {

  const connection = useConnection();
  const player = usePlayer();
  const source = useCli();
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
      {connection === 'pending' && <div className="connection-status">Connecting to MPV...</div>}
      {connection === 'error' && <div className="connection-status">Failed to start MPV. Is it installed and in your PATH?</div>}
      {connection === 'connected' && visible && <Control player={player} />}
    </main>
  );
}

export default App;