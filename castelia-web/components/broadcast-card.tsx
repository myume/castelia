import { Broadcast } from "@/lib/types";
import Link from "next/link";
import { Card, CardContent } from "./ui/card";

export const BroadcastCard = ({ broadcast }: { broadcast: Broadcast }) => {
  return (
    <Link href={`/channel/${broadcast.channel_name}`}>
      <Card className="rounded-sm h-32 flex flex-col justify-center">
        <CardContent>
          <div>
            <h2 className="font-bold truncate text-nowrap">
              {broadcast.title}
            </h2>
            <h3 className="text-sm">{broadcast.channel_name}</h3>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
};
