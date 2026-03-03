"use client";

import { useState, useEffect } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/providers/auth-provider";
import { useRouter } from "next/navigation";
import { Eye, EyeOff, Copy, Check } from "lucide-react";

export default function SettingsPage() {
  const { user, loading, accessToken } = useAuth();
  const router = useRouter();
  const [streamKey, setStreamKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [copied, setCopied] = useState(false);
  const [fetching, setFetching] = useState(false);

  useEffect(() => {
    if (!loading && !user) {
      router.replace("/login");
      return;
    }

    if (user && accessToken) {
      fetchStreamKey();
    }
  }, [user, loading, accessToken]);

  const [hostname, setHostname] = useState("localhost");

  useEffect(() => {
    if (typeof window !== "undefined") {
      setHostname(window.location.hostname);
    }
  }, []);

  const fetchStreamKey = async () => {
    setFetching(true);
    try {
      const response = await fetch("/auth/streamkey", {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      });
      if (response.ok) {
        const data = await response.json();
        setStreamKey(data.stream_key);
      }
    } catch (err) {
      console.error("Failed to fetch stream key", err);
    } finally {
      setFetching(false);
    }
  };

  const copyToClipboard = () => {
    navigator.clipboard.writeText(streamKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (loading || fetching) {
    return (
      <div className="flex justify-center items-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="container mx-auto max-w-2xl py-12 px-4 animate-in">
      <h1 className="text-4xl font-extrabold tracking-tight mb-8">Settings</h1>

      <Card className="shadow-lg border-muted/60 overflow-hidden">
        <CardHeader className="bg-muted/30 pb-6 border-b">
          <CardTitle className="text-2xl font-bold">Stream Settings</CardTitle>
          <CardDescription className="text-sm font-medium">
            Use your stream key to broadcast to your channel. Keep it secret!
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-8 pt-8 px-6 pb-8">
          <div className="space-y-3">
            <Label
              htmlFor="stream-key"
              className="text-sm font-bold uppercase tracking-wider text-muted-foreground"
            >
              Stream Key
            </Label>
            <div className="flex gap-3">
              <div className="relative flex-1 group">
                <Input
                  id="stream-key"
                  type={showKey ? "text" : "password"}
                  value={streamKey}
                  readOnly
                  className="pr-12 h-12 font-mono text-base border-muted focus-visible:ring-primary/20 bg-muted/20"
                />
                <button
                  type="button"
                  onClick={() => setShowKey(!showKey)}
                  className="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors p-1"
                >
                  {showKey ? <EyeOff size={20} /> : <Eye size={20} />}
                </button>
              </div>
              <Button
                variant="secondary"
                size="icon"
                onClick={copyToClipboard}
                title="Copy to clipboard"
                className="shrink-0 h-12 w-12 border border-muted hover:border-primary/30 transition-all shadow-sm"
              >
                {copied ? (
                  <Check className="text-green-600" size={20} />
                ) : (
                  <Copy size={20} />
                )}
              </Button>
            </div>
            <p className="text-xs text-muted-foreground italic">
              Anyone with this key can stream to your channel. Do not share it.
            </p>
          </div>

          <div className="pt-8 border-t border-muted/60">
            <div className="text-sm font-bold uppercase tracking-wider text-muted-foreground mb-3">
              Streaming Server URL
            </div>
            <div className="group relative">
              <code className="block bg-muted/40 p-4 rounded-xl text-sm font-mono break-all border border-muted/40 group-hover:border-primary/20 transition-all">
                rtmp://{hostname}/live
              </code>
              <button
                onClick={() => {
                  navigator.clipboard.writeText(`rtmp://${hostname}/live`);
                  // could add a toast here
                }}
                className="absolute right-3 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity p-2 text-muted-foreground hover:text-primary"
              >
                <Copy size={16} />
              </button>
            </div>
            <p className="text-xs text-muted-foreground mt-4 leading-relaxed">
              Copy this URL and your Stream Key into your streaming software
              (like{" "}
              <a
                href="https://obsproject.com"
                target="_blank"
                className="underline hover:text-primary"
              >
                OBS Studio
              </a>
              ) to start broadcasting.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
