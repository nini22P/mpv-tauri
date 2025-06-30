import { useEffect, useRef, useState } from 'react';
import './VideoRect.css';
import { invoke } from '@tauri-apps/api/core';
import { Connection } from '../hooks/useConnection';

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

      if (ratio.left === left && ratio.right === right && ratio.top === top && ratio.bottom === bottom) return;

      setRatio({ left, right, top, bottom });
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

  return <div ref={videoRectRef} className="video-rect"></div>;
};

export default VideoRect;
