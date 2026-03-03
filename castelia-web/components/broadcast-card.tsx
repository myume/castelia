import { Broadcast, StreamStatus } from "@/lib/types";
import Link from "next/link";
import { Card, CardContent } from "./ui/card";

export const BroadcastCard = ({ broadcast }: { broadcast: Broadcast }) => {
  return (
    <Link href={`/channel/${broadcast.channel_name}`} className="group block">
      <Card className="bg-transparent border-none shadow-none overflow-visible group-hover:scale-[1.02] transition-transform duration-200">
        <div className="relative aspect-video bg-muted/80 rounded-md overflow-hidden ring-1 ring-border shadow-sm group-hover:shadow-lg group-hover:ring-primary/40 transition-all">
          {/* Placeholder for stream thumbnail */}
          <div className="absolute inset-0 flex items-center justify-center bg-gradient-to-br from-primary/10 to-accent/20">
            <span className="text-muted-foreground/30 font-black text-6xl select-none">
              {broadcast.channel_name.charAt(0).toUpperCase()}
            </span>
          </div>

          <div className="absolute bottom-2 left-2 flex gap-1.5 items-center pointer-events-none">
            {broadcast.status === StreamStatus.Published && (
              <div className="bg-red-600 text-white text-[11px] font-bold px-1.5 py-0.5 rounded shadow-sm uppercase tracking-tighter">
                Live
              </div>
            )}
            <div className="bg-black/60 text-white/90 text-[11px] font-bold px-1.5 py-0.5 rounded shadow-sm backdrop-blur-sm">
              1.2K viewers
            </div>
          </div>
        </div>
        <CardContent className="p-2.5 pt-3">
          <div className="flex gap-3 items-start">
            <div className="size-10 rounded-full bg-primary/20 shrink-0 flex items-center justify-center font-bold text-sm border-2 border-background ring-1 ring-primary/30 text-primary">
              {broadcast.channel_name.charAt(0).toUpperCase()}
            </div>
            <div className="min-w-0 flex-1">
              <h2 className="font-bold text-sm truncate text-foreground group-hover:text-primary transition-colors leading-tight">
                {broadcast.title || "No Title"}
              </h2>
              <p className="text-xs text-muted-foreground font-semibold mt-1 hover:text-primary transition-colors cursor-pointer">
                {broadcast.channel_name}
              </p>
              <div className="flex gap-1 mt-1">
                <span className="text-[10px] bg-muted px-1.5 py-0.5 rounded-full text-muted-foreground font-bold hover:bg-accent hover:text-foreground transition-colors cursor-pointer">
                  Category
                </span>
                <span className="text-[10px] bg-muted px-1.5 py-0.5 rounded-full text-muted-foreground font-bold hover:bg-accent hover:text-foreground transition-colors cursor-pointer">
                  English
                </span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
};
