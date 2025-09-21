import { command } from "tauri-plugin-mpv-api";

export const loadFile = async (file: string) => {
  await command('loadfile', [file]);
  await play();
};

export const playlistPlay = async (index: number) => {
  await command('playlist-play-index', [index]);
};

export const playlistNext = async () => {
  await command('playlist-next');
};

export const playlistPrev = async () => {
  await command('playlist-prev');
};

export const play = async () => {
  await command('set_property', ['pause', false]);
};

export const pause = async () => {
  await command('set_property', ['pause', true]);
};

export const stop = async () => {
  await pause();
  await command('stop');
};

export const seek = async (value: number) => {
  await command('seek', [value, 'absolute']);
};

export const seekForward = async () => {
  await command('seek', ['10', 'relative']);
};

export const seekBackward = async () => {
  await command('seek', ['-10', 'relative']);
};