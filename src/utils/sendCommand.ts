import { invoke } from "@tauri-apps/api/core";

export interface MpvCommand {
  command: (string | boolean | number)[];
}

const sendCommand = async (command: MpvCommand) => {
  try {
    const commandJson = JSON.stringify(command);
    await invoke<string>('send_mpv_command', { commandJson });
  } catch (err) {
    console.error('Error sending MPV command:', err);
  }
};

export default sendCommand;