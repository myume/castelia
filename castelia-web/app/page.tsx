"use client";

import { BroadcastCard } from "@/components/broadcast-card";
import { Button } from "@/components/ui/button";
import { Broadcast } from "@/lib/types";
import { ChevronLeft, ChevronRight } from "lucide-react";
import Link from "next/link";
import { useCallback, useEffect, useState } from "react";

export default function Home() {
  const [page, setPage] = useState(`/broadcasts/list?offset=0&limit=20`);
  const [nextPage, setNextPage] = useState<string>();
  const [prevPage, setPrevPage] = useState<string>();
  const [broadcasts, setBroadcasts] = useState<Broadcast[]>([]);

  const fetchBroadcasts = useCallback(async (url: string) => {
    const response = await fetch(url);
    if (!response.ok) {
      console.log("Failed to list broadcasts");
      return;
    }

    const data = await response.json();
    setBroadcasts(data.data);
    setNextPage(data.next);
    setPrevPage(data.prev);
  }, []);

  useEffect(() => {
    fetchBroadcasts(page);
  }, [page, fetchBroadcasts]);

  return (
    <div className="container mx-auto px-4 py-8">
      {broadcasts.length > 0 ? (
        <div className="space-y-8">
          <h1 className="text-3xl font-bold tracking-tight">Live Streams</h1>
          <ul className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {broadcasts.map((broadcast) => (
              <li key={broadcast.channel_name}>
                <BroadcastCard broadcast={broadcast} />
              </li>
            ))}
          </ul>
          <div className="flex justify-center items-center gap-4 pt-4">
            <Button
              variant="outline"
              disabled={!prevPage}
              onClick={() => {
                prevPage && setPage(prevPage);
              }}
              className="gap-2"
            >
              <ChevronLeft size={16} />
              Previous
            </Button>
            <div className="text-sm text-muted-foreground font-medium">
              Page navigation
            </div>
            <Button
              variant="outline"
              disabled={!nextPage}
              onClick={() => {
                nextPage && setPage(nextPage);
              }}
              className="gap-2"
            >
              Next
              <ChevronRight size={16} />
            </Button>
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center min-h-[50vh] space-y-4">
          <h1 className="text-2xl font-semibold text-muted-foreground">
            No active broadcasts
          </h1>
          <p className="text-muted-foreground">
            Check back later or start your own stream!
          </p>
          <Button asChild>
            <Link href="/live">Go Live Now</Link>
          </Button>
        </div>
      )}
    </div>
  );
}
