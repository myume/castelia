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
import { Broadcast, StreamStatus } from "@/lib/types";

export default function GoLivePage() {
  const { user, loading, accessToken } = useAuth();
  const router = useRouter();
  const [broadcast, setBroadcast] = useState<Broadcast>();
  const [error, setError] = useState("");

  const fetchBroadcast = async () => {
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
    fetchBroadcast();
  }, [user]);

  const handleSubmit = async (status?: StreamStatus) => {
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

    if (!status) {
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
    <div className="container mx-auto px-4 py-12 flex justify-center items-center min-h-[calc(100vh-64px)]">
      <Card className="w-full max-w-lg shadow-lg border-muted">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl font-bold tracking-tight text-center">
            Start Your Stream
          </CardTitle>
          <CardDescription className="text-center">
            Configure your broadcast details before going live.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="title" className="font-semibold">
                Broadcast Title
              </Label>
              <Input
                id="title"
                value={broadcast?.title ?? ""}
                onChange={(e) =>
                  setBroadcast((cast) => {
                    if (!cast) {
                      return {
                        title: e.target.value,
                        channel_name: user?.username || "",
                        status: StreamStatus.Offline,
                        private: false,
                      };
                    }

                    return { ...cast, title: e.target.value };
                  })
                }
                placeholder="My awesome stream"
                className="h-11"
              />
            </div>
            {broadcast?.status === StreamStatus.Offline && (
              <div className="p-3 bg-primary/10 border border-primary/20 rounded-md text-sm text-primary font-medium animate-in">
                Your stream is currently offline. Please start your broadcast in
                your streaming software (like OBS) first.
              </div>
            )}
            <div className="flex items-center gap-3 p-4 bg-muted/30 rounded-lg border border-muted/50">
              <Input
                id="private"
                type="checkbox"
                checked={broadcast?.private || false}
                onChange={(e) =>
                  setBroadcast((cast) => {
                    if (!cast) {
                      return cast;
                    }

                    return { ...cast, private: e.target.checked };
                  })
                }
                className="size-5 rounded cursor-pointer"
              />
              <div className="grid gap-1">
                <Label
                  htmlFor="private"
                  className="font-medium cursor-pointer leading-none"
                >
                  Private broadcast
                </Label>
                <p className="text-xs text-muted-foreground">
                  Only people with the link can view your stream.
                </p>
              </div>
            </div>
          </div>
          {error && (
            <div className="bg-destructive/10 text-destructive text-sm p-3 rounded-md border border-destructive/20">
              {error}
            </div>
          )}
        </CardContent>
        <CardFooter className="flex flex-col gap-3 pt-2">
          <div className="flex gap-3 w-full">
            <Button
              variant="outline"
              onClick={() => handleSubmit()}
              className="flex-1 h-11"
            >
              Save Settings
            </Button>
            <Button
              disabled={broadcast?.status === StreamStatus.Offline}
              className={`flex-1 h-11 font-semibold transition-all duration-300 ${
                broadcast?.status === StreamStatus.Unpublished
                  ? "bg-green-600 hover:bg-green-700 text-white"
                  : broadcast?.status === StreamStatus.Published
                    ? "bg-red-600 hover:bg-red-700 text-white"
                    : ""
              }`}
              onClick={() =>
                handleSubmit(
                  broadcast?.status === StreamStatus.Unpublished
                    ? StreamStatus.Published
                    : StreamStatus.Unpublished,
                )
              }
            >
              {broadcast?.status === StreamStatus.Unpublished
                ? "Go Live"
                : broadcast?.status === StreamStatus.Published
                  ? "End Stream"
                  : "Offline"}
            </Button>
          </div>
          <p className="text-[10px] text-muted-foreground text-center">
            Make sure your streaming software is configured with your stream
            key.
          </p>
        </CardFooter>
      </Card>
    </div>
  );
}
