import { useEffect, useRef } from 'react';
import './VideoRect.css';
import { invoke } from '@tauri-apps/api/core';

const VideoRect = () => {
  const videoRectRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const videoRect = videoRectRef.current;

    if (!videoRect) return;

    const updateVideoPanAndZoom = () => {
      const rect = videoRect.getBoundingClientRect();

      invoke('set_video_margin_ratio', {
        ratio: {
          left: Math.round(rect.left) / window.innerWidth,
          right: 1 - (Math.round(rect.right) / window.innerWidth),
          top: Math.round(rect.top) / window.innerHeight,
          bottom: 1 - (Math.round(rect.bottom) / window.innerHeight),
        }
      });
    };

    const throttledUpdate = () => window.requestAnimationFrame(updateVideoPanAndZoom);

    const resizeObserver = new ResizeObserver(throttledUpdate);
    resizeObserver.observe(videoRect);

    throttledUpdate();

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  return <div ref={videoRectRef} className="video-rect"></div>;
};

export default VideoRect;
