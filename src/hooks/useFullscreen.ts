import { getCurrentWindow } from "@tauri-apps/api/window";
import { useState } from "react";

const useFullscreen = () => {
  const [isFullscreen, setIsFullscreen] = useState(false);

  const toggleFullscreen = async () => {
    await getCurrentWindow().setFullscreen(!isFullscreen);
    setIsFullscreen(!isFullscreen);
  };

  return {
    isFullscreen,
    toggleFullscreen,
  };
};

export default useFullscreen;

