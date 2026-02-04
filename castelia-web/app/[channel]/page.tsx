"use client";

import { use, useCallback, useEffect, useRef, useState } from "react";
import Hls from "hls.js";
import { Broadcast, StreamStatus } from "@/lib/types";

export default function PlayerPage({
  params,
}: {
  params: Promise<{ channel: string }>;
}) {
  const { channel } = use(params);
  const [broadcast, setBroadcast] = useState<Broadcast>();
  const videoRef = useRef<HTMLVideoElement>(null);

  const fetchBroadcast = useCallback(async () => {
    const response = await fetch(`/broadcasts/${channel}`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      credentials: "include",
    });

    if (!response.ok) {
      console.error("Failed to fetch broadcast for user");
      return;
    }

    setBroadcast(await response.json());
  }, [channel]);

  useEffect(() => {
    fetchBroadcast();
  }, [fetchBroadcast]);

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
  }, [channel, broadcast?.status]);

  if (!broadcast || broadcast.private) {
    return <div>Broadcast not found</div>;
  }

  return (
    <div>
      {broadcast.status !== StreamStatus.Published ? (
        <div>Broadcast is offline</div>
      ) : (
        <video ref={videoRef} controls></video>
      )}
      <div className="p-2">
        <h1 className="text-bold">{broadcast.title}</h1>
        <h2>{broadcast.channel_name}</h2>
        <h2>started: {broadcast.start_time}</h2>
      </div>
    </div>
  );
}
