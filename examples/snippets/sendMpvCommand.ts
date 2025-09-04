import { sendMpvCommand } from 'tauri-plugin-mpv-api';

// Load file
await sendMpvCommand({ command: ['loadfile', '/path/to/video.mp4'] });

// Play/pause
await sendMpvCommand({ command: ['set_property', 'pause', false] });
await sendMpvCommand({ command: ['set_property', 'pause', true] });

// Seek to position
await sendMpvCommand({ command: ['seek', 30, 'absolute'] });
await sendMpvCommand({ command: ['seek', 10, 'relative'] });

// Set volume
await sendMpvCommand({ command: ['set_property', 'volume', 80] });

// Get property
const duration = await sendMpvCommand({ command: ['get_property', 'duration'] });
console.log('Duration:', duration.data);