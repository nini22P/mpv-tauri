import { open } from '@tauri-apps/plugin-dialog';
import './Control.css';
import { PlayerStatus, sendCommand } from '../App';
import { getCurrentWindow } from '@tauri-apps/api/window';

const formatTime = (seconds: number | null | undefined): string => {
  if (seconds === null || seconds === undefined || isNaN(seconds)) {
    return "00:00";
  }
  const flooredSeconds = Math.floor(seconds);
  const m = Math.floor(flooredSeconds / 60);
  const s = flooredSeconds % 60;
  return `${m < 10 ? '0' : ''}${m}:${s < 10 ? '0' : ''}${s}`;
};

const Control = ({ playerStatus }: { playerStatus: PlayerStatus }) => {
  const handlePlay = () => {
    sendCommand({ command: ['set_property', 'pause', false] });
  };

  const handlePause = () => {
    sendCommand({ command: ['set_property', 'pause', true] });
  };

  const handleStop = () => {
    sendCommand({ command: ['stop'] });
  };

  const handleSeekForward = () => {
    sendCommand({ command: ['seek', '10', 'relative'] });
  };

  const handleSeekBackward = () => {
    sendCommand({ command: ['seek', '-10', 'relative'] });
  };

  const handleLoadFile = async () => {
    const file = await open({
      multiple: false,
      directory: false,
    });

    if (file) {
      sendCommand({ command: ['loadfile', file as string] });
    }
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    sendCommand({ command: ['seek', e.target.value, 'absolute'] });
  };

  const handleToggleFullscreen = async () => await getCurrentWindow().setFullscreen(!await getCurrentWindow().isFullscreen());

  return (
    <div className="control">
      <div className="control-buttons">
        <button type="button" onClick={handleLoadFile} >Load File</button>
        <button
          type="button"
          onClick={
            playerStatus.isPaused
              ? () => handlePlay()
              : () => handlePause()
          }
        >
          {playerStatus.isPaused ? 'Play' : 'Pause'}
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
        max={playerStatus.duration}
        value={playerStatus.timePos}
        step={1}
        onChange={handleSeek}
      />
      <p className="time"> {formatTime(playerStatus.timePos)} / {formatTime(playerStatus.duration)}</p>
    </div>
  );
};

export default Control;