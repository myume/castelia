"use client";

import { Broadcast } from "@/lib/types";
import Link from "next/link";
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
        <ul className="grid grid-cols-3 gap-2 w-screen p-5">
          {broadcasts.map((broadcast) => (
            <li>
              <Link href={`/channel/${broadcast.channel_name}`}>
                <div>
                  <h2>{broadcast.title}</h2>
                  <h3>{broadcast.channel_name}</h3>
                </div>
              </Link>
            </li>
          ))}
        </ul>
      ) : (
        <h1>There are no broadcasts</h1>
      )}
    </div>
  );
}
