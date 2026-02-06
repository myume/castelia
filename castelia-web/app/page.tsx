"use client";

import { BroadcastCard } from "@/components/broadcast-card";
import { Broadcast, StreamStatus } from "@/lib/types";
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
        <ul className="grid grid-cols-3 gap-4 w-screen h-screen p-10">
          {broadcasts.map((broadcast) => (
            <li key={broadcast.channel_name}>
              <BroadcastCard broadcast={broadcast} />
            </li>
          ))}
        </ul>
      ) : (
        <h1>There are no broadcasts</h1>
      )}
    </div>
  );
}
