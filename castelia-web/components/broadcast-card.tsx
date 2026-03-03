import { Broadcast, StreamStatus } from "@/lib/types";
import Link from "next/link";

export const BroadcastCard = ({ broadcast }: { broadcast: Broadcast }) => {
  return (
    <Link href={`/channel/${broadcast.channel_name}`} className="group block">
      <div className="flex flex-col gap-3 group-hover:scale-[1.02] transition-all duration-200">
        <div className="relative aspect-video bg-muted/80 rounded-xl overflow-hidden ring-1 ring-border/5 shadow-sm group-hover:shadow-xl group-hover:ring-primary/40 transition-all">
          {/* Placeholder for stream thumbnail */}
          <div className="absolute inset-0 flex items-center justify-center bg-linear-to-br from-primary/10 to-accent/20">
            <span className="text-muted-foreground/30 font-black text-6xl select-none group-hover:scale-110 transition-transform duration-500">
              {broadcast.channel_name.charAt(0).toUpperCase()}
            </span>
          </div>

          <div className="absolute top-2 left-2 flex gap-1.5 items-center pointer-events-none">
            {broadcast.status === StreamStatus.Published && (
              <div className="bg-red-600 text-white text-[11px] font-black px-2 py-0.5 rounded-sm shadow-xl uppercase tracking-widest animate-pulse border border-red-500/50">
                Live
              </div>
            )}
          </div>
        </div>
        
        <div className="flex gap-3 items-start px-1">
          <div className="size-10 rounded-full bg-linear-to-br from-primary/10 to-accent/10 shrink-0 flex items-center justify-center font-black text-sm border-2 border-background ring-1 ring-primary/20 text-primary shadow-sm group-hover:ring-primary/40 transition-all">
            {broadcast.channel_name.charAt(0).toUpperCase()}
          </div>
          <div className="min-w-0 flex-1">
            <h2 className="font-bold text-sm truncate text-foreground group-hover:text-primary transition-colors leading-tight">
              {broadcast.title}
            </h2>
            <p className="text-xs text-muted-foreground font-bold mt-1 group-hover:text-foreground transition-colors">
              {broadcast.channel_name}
            </p>
          </div>
        </div>
      </div>
    </Link>
  );
};
