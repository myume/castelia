"use client";

import { use, useCallback, useEffect, useRef, useState } from "react";
import Hls from "hls.js";
import { Broadcast, StreamStatus } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Heart, Share2, MoreHorizontal, AlertCircle } from "lucide-react";
import Link from "next/link";

export default function PlayerPage({
  params,
}: {
  params: Promise<{ channel: string }>;
}) {
  const { channel } = use(params);
  const [broadcast, setBroadcast] = useState<Broadcast>();
  const [error, setError] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  const fetchBroadcast = useCallback(async () => {
    try {
      const response = await fetch(`/broadcasts/${channel}`, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        credentials: "include",
      });

      if (!response.ok) {
        setError(true);
        return;
      }

      setBroadcast(await response.json());
    } catch (err) {
      console.error("Failed to fetch broadcast", err);
      setError(true);
    }
  }, [channel]);

  useEffect(() => {
    fetchBroadcast();
  }, [fetchBroadcast]);

  useEffect(() => {
    if (broadcast?.status !== StreamStatus.Published) return;

    const video = videoRef.current;
    if (video) {
      if (Hls.isSupported()) {
        const hls = new Hls();
        hls.loadSource(`/broadcasts/hls/${channel}/stream.m3u8`);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          video.play().catch((e) => console.log("Autoplay prevented", e));
        });

        return () => hls.destroy();
      } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
        video.src = `/broadcasts/hls/${channel}/stream.m3u8`;
        video.addEventListener("loadedmetadata", () => {
          video.play().catch((e) => console.log("Autoplay prevented", e));
        });
      }
    }
  }, [channel, broadcast?.status]);

  if (error || (broadcast && broadcast.private)) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[50vh] space-y-4 animate-in">
        <AlertCircle size={48} className="text-muted-foreground" />
        <h1 className="text-2xl font-bold">Broadcast not found</h1>
        <p className="text-muted-foreground">
          This channel might be private or doesn&apos;t exist.
        </p>
        <Button variant="outline" asChild>
          <Link href="/">Go Back Home</Link>
        </Button>
      </div>
    );
  }

  if (!broadcast) {
    return (
      <div className="flex justify-center items-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="bg-background min-h-[calc(100vh-64px)]">
      <div className="container mx-auto px-4 py-6 max-w-7xl animate-in flex flex-col gap-6">
        <div className="relative aspect-video bg-black rounded-xl overflow-hidden shadow-2xl group border border-border/50">
          {broadcast.status !== StreamStatus.Published ? (
            <div className="absolute inset-0 flex flex-col items-center justify-center bg-zinc-900">
              <div className="size-24 rounded-full bg-primary/10 flex items-center justify-center mb-6 border border-primary/20">
                <span className="text-primary font-black text-5xl italic select-none">
                  C
                </span>
              </div>
              <h2 className="text-3xl font-black tracking-tight text-white uppercase italic">
                {broadcast.channel_name} is Offline
              </h2>
              <p className="text-zinc-400 mt-2 font-semibold">
                The broadcast has ended. Check back later!
              </p>
            </div>
          ) : (
            <video
              ref={videoRef}
              className="w-full h-full object-contain"
              controls
              autoPlay
              playsInline
            />
          )}

          {broadcast.status === StreamStatus.Published && (
            <div className="absolute top-4 left-4 bg-red-600 text-white text-[12px] font-black px-2.5 py-1 rounded-sm shadow-xl uppercase tracking-widest animate-pulse pointer-events-none border border-red-500/50">
              Live
            </div>
          )}
        </div>

        <div className="flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between bg-card p-6 rounded-xl border border-border/50 shadow-sm">
          <div className="flex gap-4 items-center min-w-0">
            <div className="size-16 rounded-full bg-linear-to-br from-primary/20 to-accent/20 shrink-0 flex items-center justify-center font-black text-2xl border-4 border-background ring-2 ring-primary/20 text-primary shadow-lg overflow-hidden">
              {broadcast.channel_name.charAt(0).toUpperCase()}
            </div>
            <div className="min-w-0">
              <h1 className="text-2xl font-black tracking-tight text-foreground truncate leading-none mb-2">
                {broadcast.title || "Untitled Stream"}
              </h1>
              <div className="flex items-center gap-2">
                <p className="text-sm font-bold text-primary hover:underline cursor-pointer transition-all">
                  {broadcast.channel_name}
                </p>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2 w-full sm:w-auto">
            <Button className="flex-1 sm:flex-none gap-2 font-black uppercase tracking-tight h-10 px-6 bg-primary hover:bg-primary/90 transition-all active:scale-95 shadow-lg shadow-primary/20">
              <Heart size={16} fill="currentColor" />
              Follow
            </Button>
            <Button
              variant="outline"
              size="icon"
              className="shrink-0 size-10 bg-background/50 border-border/50 hover:bg-accent hover:text-accent-foreground transition-all active:scale-95"
            >
              <Share2 size={18} />
            </Button>
            <Button
              variant="outline"
              size="icon"
              className="shrink-0 size-10 bg-background/50 border-border/50 hover:bg-accent hover:text-accent-foreground transition-all active:scale-95"
            >
              <MoreHorizontal size={18} />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
