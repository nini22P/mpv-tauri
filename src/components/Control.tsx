import { open } from '@tauri-apps/plugin-dialog';
import './Control.css';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Player } from '../hooks/usePlayer';
import sendCommand from '../utils/sendCommand';
import formatTime from '../utils/formatTime';

const Control = ({ player }: { player: Player }) => {

  const handlePlay = async () => {
    if (player.eofReached && player.currentFile) {
      await handleSeek(0);
    }
    await sendCommand({ command: ['set_property', 'pause', false] });
  };

  const handlePause = async () => {
    await sendCommand({ command: ['set_property', 'pause', true] });
  };

  const handleStop = async () => {
    await sendCommand({ command: ['stop'] });
  };

  const handleSeekForward = async () => {
    await sendCommand({ command: ['seek', '10', 'relative'] });
  };

  const handleSeekBackward = async () => {
    await sendCommand({ command: ['seek', '-10', 'relative'] });
  };

  const handleLoadFile = async () => {
    const file = await open({
      multiple: false,
      directory: false,
    });

    if (file) {
      await sendCommand({ command: ['loadfile', file as string] });
    }
  };

  const handleSeek = async (value: number) => {
    await sendCommand({ command: ['seek', value, 'absolute'] });
  };

  const handleToggleFullscreen = async () => await getCurrentWindow().setFullscreen(!await getCurrentWindow().isFullscreen());

  return (
    <div className="control">
      <div className="control-buttons">
        <button type="button" onClick={handleLoadFile} >Load File</button>
        <button
          type="button"
          onClick={player.isPaused ? handlePlay : handlePause}
        >
          {player.isPaused ? 'Play' : 'Pause'}
        </button>
        <button type="button" onClick={handleStop} >Stop</button>
        <button type="button" onClick={handleSeekBackward} >-10s</button>
        <button type="button" onClick={handleSeekForward} >+10s</button>
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
        onChange={(e) => handleSeek(Number(e.target.value))}
      />
      <p className="time"> {formatTime(player.timePos)} / {formatTime(player.duration)}</p>
    </div>
  );
};

export default Control;