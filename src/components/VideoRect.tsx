import { useEffect, useRef, useState } from 'react';
import './VideoRect.css';
import { invoke } from '@tauri-apps/api/core';
import { Connection } from '../hooks/usePlayer';

const VideoRect = ({ connection }: { connection: Connection }) => {
  const videoRectRef = useRef<HTMLDivElement>(null);
  const [ratio, setRatio] = useState({ left: 0, right: 0, top: 0, bottom: 0 });

  useEffect(() => {
    const videoRect = videoRectRef.current;

    if (!videoRect) return;

    const updateRatio = () => {
      const rect = videoRect.getBoundingClientRect();

      const left = Math.round(rect.left) / window.innerWidth;
      const right = 1 - (Math.round(rect.right) / window.innerWidth);
      const top = Math.round(rect.top) / window.innerHeight;
      const bottom = 1 - (Math.round(rect.bottom) / window.innerHeight);

      setRatio(
        value => {
          if (value.left === left && value.right === right && value.top === top && value.bottom === bottom) return value;
          return { left, right, top, bottom }
        }
      );
    };

    const throttledUpdate = () => window.requestAnimationFrame(updateRatio);

    const resizeObserver = new ResizeObserver(throttledUpdate);
    resizeObserver.observe(videoRect);

    throttledUpdate();

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  useEffect(() => {
    if (connection !== 'connected') return;

    const updateVideoMarginRatio = async () => await invoke('set_video_margin_ratio', { ratio });

    updateVideoMarginRatio();
  }, [ratio, connection]);

  return (
    <div ref={videoRectRef} className="video-rect" style={{ backgroundColor: connection === 'connected' ? 'transparent' : 'black' }}>
      {connection === 'error' && 'MPV connection timeout. Is mpv installed and in your PATH?'}
    </div>
  );
};

export default VideoRect;
