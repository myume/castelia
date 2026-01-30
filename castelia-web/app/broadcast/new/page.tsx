"use client";

import { useState, useRef, useEffect } from "react";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import Hls from "hls.js";
import { useAuth } from "@/providers/auth-provider";
import { useRouter } from "next/navigation";

enum StreamStatus {
  Offline = "offline",
  Unpublished = "unpublished",
  Published = "published",
}

interface Broadcast {
  channel_name: string;
  title: string;
  start_time?: Date;
  status: StreamStatus;
  private: boolean;
}

export default function GoLivePage() {
  const [title, setTitle] = useState("");
  const [isPrivate, setIsPrivate] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const { user } = useAuth();
  const router = useRouter();

  if (!user) {
    router.replace("/login");
  }

  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      if (Hls.isSupported()) {
        const hls = new Hls();
        hls.loadSource(`/broadcasts/hls/${user?.username}/stream.m3u8`);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          video.play();
        });
      } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
        video.src = `/broadcasts/hls/${user?.username}/stream.m3u8`;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
      }
    }
  }, [user]);

  const handleSubmit = (status: StreamStatus) => {
    const broadcast: Omit<Broadcast, "channel_name" | "start_time"> = {
      title,
      status,
      private: isPrivate,
    };

    console.log("Submitting broadcast:", broadcast);
    alert(`Broadcast submitted with status: ${status}`);
  };

  return (
    <div className="flex gap-8">
      <div className="flex flex-col gap-4">
        <h2 className="text-2xl font-bold">
          Stream Preview (Channel: {user?.username})
        </h2>
        <video
          ref={videoRef}
          controls
          className="w-full aspect-video bg-black"
        ></video>
      </div>
      <div className="flex justify-center items-center">
        <Card className="w-full max-w-lg">
          <CardHeader>
            <CardTitle>Create a new broadcast</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4">
              <div className="grid gap-2">
                <Label htmlFor="title">Title</Label>
                <Input
                  id="title"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="My awesome stream"
                />
              </div>
              <div className="flex items-center gap-2">
                <Input
                  id="private"
                  type="checkbox"
                  checked={isPrivate}
                  onChange={(e) => setIsPrivate(e.target.checked)}
                  className="size-4"
                />
                <Label htmlFor="private">Private broadcast</Label>
              </div>
            </div>
          </CardContent>
          <CardFooter className="flex justify-end gap-2">
            <Button
              variant="outline"
              onClick={() => handleSubmit(StreamStatus.Unpublished)}
            >
              Save as Unpublished
            </Button>
            <Button onClick={() => handleSubmit(StreamStatus.Published)}>
              Go Live
            </Button>
          </CardFooter>
        </Card>
      </div>
    </div>
  );
}

