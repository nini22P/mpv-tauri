import './App.css'
import useCli from './hooks/useCli'
import Player from './pages/Player'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from '@tauri-apps/api/window'
import usePlayerStore from './store'

function App() {
  const source = useCli()
  const integrationMode = usePlayerStore.use.integrationMode()
  const updatePlayerState = usePlayerStore.use.updatePlayerState()

  const createPlayerWindow = () => {
    const windowLabel = getCurrentWindow().label
    new WebviewWindow(windowLabel + '_' + 1, {
      width: 1280,
      height: 720,
      transparent: true,
      center: true,
    })
  }

  return (
    <main className="app">
      <div style={{ position: 'fixed', top: 0, left: 0, display: 'flex', gap: '0.25rem', padding: '0.25rem' }}>
        <button onClick={createPlayerWindow} >Create New Window</button>
        <button
          title={'Switch Integration Mode'}
          onClick={
            async () => {
              updatePlayerState('integrationMode', integrationMode === 'wid' ? 'render' : 'wid')
              updatePlayerState('timePos', 0)
              updatePlayerState('duration', 0)
              updatePlayerState('filename', null)
            }
          }
        >
          {integrationMode}
        </button>
      </div>

      <Player source={source} />
    </main>
  )
}

export default App