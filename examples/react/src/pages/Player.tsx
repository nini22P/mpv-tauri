import { useEffect } from "react";
import useAutoHide from "../hooks/useAutoHide";
import { getCurrentWindow } from "@tauri-apps/api/window";
import VideoRect from "./VideoRect";
import Controls from "./Controls";
import './Player.css';
import usePlayerStore from "../store";
import useMpv from "../hooks/useMpv";
import { loadFile } from "../utils/commands";

const Player = ({ source }: { source: string | null }) => {
  useMpv();

  const { visible, show: showControls, hide: hideControls } = useAutoHide(5000);

  const connection = usePlayerStore.use.connection();
  const currentFile = usePlayerStore.use.currentFile();
  const isFullscreen = usePlayerStore.use.isFullscreen();

  useEffect(() => {
    if (connection === 'connected') {
      showControls();
    }
  }, [connection]);

  useEffect(() => {
    if (connection === 'connected' && source) {
      loadFile(source);
    }
  }, [connection, source])

  useEffect(() => {
    getCurrentWindow().setTitle(currentFile ? `${currentFile} - mpv-tauri` : 'mpv-tauri');
  }, [currentFile])

  return (
    <div className="player" onMouseMove={showControls} onMouseLeave={hideControls}>
      <VideoRect connection={connection} />
      {
        isFullscreen
          ? visible && <div style={{ position: 'fixed', left: 0, right: 0, bottom: 0 }}>
            <Controls />
          </div>
          : <Controls />
      }
    </div>
  );
}

export default Player;