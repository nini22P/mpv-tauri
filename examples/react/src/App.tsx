import "./App.css";
import useCli from "./hooks/useCli";
import Player from "./components/Player";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";

function App() {
  const source = useCli();

  const createPlayerWindow = () => {
    const windowLabel = getCurrentWindow().label
    new WebviewWindow(windowLabel + "_" + 1, {
      width: 1280,
      height: 720,
      transparent: true,
      center: true,
    });
  };

  return (
    <main className="app">
      <button onClick={createPlayerWindow} style={{ position: 'fixed', top: 0, left: 0 }}>Create New Window</button>
      <Player source={source} />
    </main>
  );
}

export default App;