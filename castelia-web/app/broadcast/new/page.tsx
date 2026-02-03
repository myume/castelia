"use client";

import { useState, useEffect } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
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
  const { user, loading, accessToken } = useAuth();
  const router = useRouter();
  const [broadcast, setBroadcast] = useState<Broadcast>();
  const [error, setError] = useState("");

  const fetchBroacast = async () => {
    const response = await fetch(`/broadcasts/${user?.username}`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Authorization: `Bearer ${accessToken}`,
      },
      credentials: "include",
    });

    if (!response.ok) {
      console.error("Failed to fetch broadcast for user");
      return;
    }

    setBroadcast(await response.json());
  };

  useEffect(() => {
    if (!user && !loading) {
      router.replace("/login");
      return;
    }
    fetchBroacast();
  }, [user]);

  const handleSubmit = async (status: StreamStatus) => {
    setError("");
    const update = await fetch(`/broadcasts/${user?.username}`, {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Authorization: `Bearer ${accessToken}`,
      },
      body: JSON.stringify(broadcast),
      credentials: "include",
    });

    if (!update.ok) {
      console.error("Failed to update broadcast");
      return;
    }

    let response;
    if (status == StreamStatus.Published) {
      response = await fetch(`/broadcasts/${user?.username}/publish`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
        credentials: "include",
      });
    } else {
      response = await fetch(`/broadcasts/${user?.username}/unpublish`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
        credentials: "include",
      });
    }

    if (!response.ok) {
      console.error("Failed to update stream status");
      setError(await response.text());
    }

    setBroadcast((broadcast) =>
      broadcast ? { ...broadcast, status } : broadcast,
    );
  };

  return (
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
                value={broadcast?.title}
                onChange={(e) =>
                  setBroadcast((cast) => {
                    if (!cast) {
                      return cast;
                    }

                    return { ...cast, title: e.target.value };
                  })
                }
                placeholder="My awesome stream"
              />
            </div>
            <div className="flex items-center gap-2">
              <Input
                id="private"
                type="checkbox"
                checked={broadcast?.private}
                onChange={(e) =>
                  setBroadcast((cast) => {
                    if (!cast) {
                      return cast;
                    }

                    return { ...cast, private: e.target.checked };
                  })
                }
                className="size-4"
              />
              <Label htmlFor="private">Private broadcast</Label>
            </div>
          </div>
          {error && (
            <CardDescription className="text-red-500 mt-5">
              {error}
            </CardDescription>
          )}
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
  );
}
