import { command } from "tauri-plugin-mpv-api";

export const loadFile = async (file: string) => {
  await command({ command: ['loadfile', file] });
  await play();
};

export const playlistPlay = async (index: number) => {
  await command({ command: ['playlist-play-index', index] });
};

export const playlistNext = async () => {
  await command({ command: ['playlist-next'] });
};

export const playlistPrev = async () => {
  await command({ command: ['playlist-prev'] });
};

export const play = async () => {
  await command({ command: ['set_property', 'pause', false] });
};

export const pause = async () => {
  await command({ command: ['set_property', 'pause', true] });
};

export const stop = async () => {
  await pause();
  await command({ command: ['stop'] });
};

export const seek = async (value: number) => {
  await command({ command: ['seek', value, 'absolute'] });
};

export const seekForward = async () => {
  await command({ command: ['seek', '10', 'relative'] });
};

export const seekBackward = async () => {
  await command({ command: ['seek', '-10', 'relative'] });
};