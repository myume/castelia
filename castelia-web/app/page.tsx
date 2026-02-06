"use client";

import { BroadcastCard } from "@/components/broadcast-card";
import { Button } from "@/components/ui/button";
import { Broadcast } from "@/lib/types";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

export default function Home() {
  const [page, setPage] = useState(`/broadcasts?offset=0&limit=20`);
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
    <div>
      {broadcasts.length > 0 ? (
        <div className="w-screen p-10">
          <ul className="grid grid-cols-3 gap-4 w-full h-screen">
            {broadcasts.map((broadcast) => (
              <li key={broadcast.channel_name}>
                <BroadcastCard broadcast={broadcast} />
              </li>
            ))}
          </ul>
          <div className="flex justify-center gap-5">
            <Button
              disabled={!prevPage}
              onClick={() => {
                prevPage && setPage(prevPage);
              }}
            >
              <ChevronLeft />
              Prev
            </Button>
            <Button
              disabled={!nextPage}
              onClick={() => {
                nextPage && setPage(nextPage);
              }}
            >
              Next
              <ChevronRight />
            </Button>
          </div>
        </div>
      ) : (
        <h1>There are no broadcasts</h1>
      )}
    </div>
  );
}
