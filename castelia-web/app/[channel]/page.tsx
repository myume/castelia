"use client";

import { use, useEffect, useRef } from "react";
import Hls from "hls.js";

export default function PlayerPage({
  params,
}: {
  params: Promise<{ channel: string }>;
}) {
  const { channel } = use(params);
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      if (Hls.isSupported()) {
        const hls = new Hls();
        hls.loadSource(`/broadcasts/hls/${channel}/stream.m3u8`);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          video.play();
        });
      } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
        video.src = `/broadcasts/hls/${channel}/stream.m3u8`;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
      }
    }
  }, [channel]);

  return (
    <div>
      <video ref={videoRef} controls></video>
    </div>
  );
}
