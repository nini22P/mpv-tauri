import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import './control.css'

interface MpvCommand {
  command: string[];
}

const Control = () => {
  const sendCommand = async (command: MpvCommand) => {
    try {
      const commandJson = JSON.stringify(command);
      await invoke<string>('send_mpv_command', { commandJson });
    } catch (err) {
      console.error('Error sending MPV command:', err);
    }
  };

  const handlePlay = () => {
    sendCommand({ command: ['set_property', 'pause', false as unknown as string] });
  };

  const handlePause = () => {
    sendCommand({ command: ['set_property', 'pause', true as unknown as string] });
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

  const handleVolumeUp = () => {
    sendCommand({ command: ['add', 'volume', '5'] });
  };

  const handleVolumeDown = () => {
    sendCommand({ command: ['add', 'volume', '-5'] });
  };

  const handleMute = () => {
    sendCommand({ command: ['cycle', 'mute'] });
  };

  const handleLoadFile = async () => {
    const file = await open({
      multiple: false,
      directory: false,
    });

    if (file) {
      sendCommand({ command: ['loadfile', file] });
    }
  };

  return (
    <div className="control">
      <div className="control-buttons">
        <button type="button" onClick={handlePlay} >Play</button>
        <button type="button" onClick={handlePause} >Pause</button>
        <button type="button" onClick={handleStop} >Stop</button>
        <button type="button" onClick={handleSeekBackward} >-10s</button>
        <button type="button" onClick={handleSeekForward} >+10s</button>
        <button type="button" onClick={handleVolumeDown} >Vol-</button>
        <button type="button" onClick={handleVolumeUp} >Vol+</button>
        <button type="button" onClick={handleMute} >Mute</button>
        <button type="button" onClick={handleLoadFile} >Load File</button>
      </div>
    </div>
  );
};

export default Control;
