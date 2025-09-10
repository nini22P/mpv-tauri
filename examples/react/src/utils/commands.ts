import { sendMpvCommand } from "tauri-plugin-mpv-api";

export const loadFile = async (file: string) => {
  await sendMpvCommand({ command: ['loadfile', file] });
  await play();
};

export const playlistPlay = async (index: number) => {
  await sendMpvCommand({ command: ['playlist-play-index', index] });
};

export const playlistNext = async () => {
  await sendMpvCommand({ command: ['playlist-next'] });
};

export const playlistPrev = async () => {
  await sendMpvCommand({ command: ['playlist-prev'] });
};

export const play = async () => {
  await sendMpvCommand({ command: ['set_property', 'pause', false] });
};

export const pause = async () => {
  await sendMpvCommand({ command: ['set_property', 'pause', true] });
};

export const stop = async () => {
  await pause();
  await sendMpvCommand({ command: ['stop'] });
};

export const seek = async (value: number) => {
  await sendMpvCommand({ command: ['seek', value, 'absolute'] });
};

export const seekForward = async () => {
  await sendMpvCommand({ command: ['seek', '10', 'relative'] });
};

export const seekBackward = async () => {
  await sendMpvCommand({ command: ['seek', '-10', 'relative'] });
};