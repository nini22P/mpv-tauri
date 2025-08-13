import { useEffect, useRef } from 'react';
import './VideoRect.css';
import { invoke } from '@tauri-apps/api/core';
import { Connection } from '../hooks/usePlayer';

interface VideoMarginRatio {
  left?: number,
  right?: number,
  top?: number,
  bottom?: number,
};

const VideoRect = ({ connection }: { connection: Connection }) => {
  const videoRectRef = useRef<HTMLDivElement>(null);
  const prevRatioRef = useRef<VideoMarginRatio>({ left: 0, right: 0, top: 0, bottom: 0 });

  useEffect(() => {
    const videoRect = videoRectRef.current;

    if (!videoRect || connection !== 'connected') return;

    const updateRatio = async () => {
      const rect = videoRect.getBoundingClientRect();

      const left = Math.round(rect.left) / window.innerWidth;
      const right = 1 - (Math.round(rect.right) / window.innerWidth);
      const top = Math.round(rect.top) / window.innerHeight;
      const bottom = 1 - (Math.round(rect.bottom) / window.innerHeight);

      const changedRatio: VideoMarginRatio = {};
      if (left !== prevRatioRef.current.left) {
        changedRatio.left = left;
      }
      if (right !== prevRatioRef.current.right) {
        changedRatio.right = right;
      }
      if (top !== prevRatioRef.current.top) {
        changedRatio.top = top;
      }
      if (bottom !== prevRatioRef.current.bottom) {
        changedRatio.bottom = bottom;
      }

      if (Object.keys(changedRatio).length > 0) {
        await invoke('set_video_margin_ratio', { ratio: changedRatio });
      }

      prevRatioRef.current = { left, right, top, bottom };
    };

    const throttledUpdate = () => window.requestAnimationFrame(updateRatio);

    const resizeObserver = new ResizeObserver(throttledUpdate);
    resizeObserver.observe(videoRect);

    throttledUpdate();

    return () => {
      resizeObserver.disconnect();
    };
  }, [connection]);

  return (
    <div ref={videoRectRef} className="video-rect" style={{ backgroundColor: connection === 'connected' ? 'transparent' : 'black' }}>
      {connection === 'error' && 'MPV connection timeout. Is mpv installed and in your PATH?'}
    </div>
  );
};

export default VideoRect;
