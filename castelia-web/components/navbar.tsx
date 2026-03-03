"use client";
import Link from "next/link";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  NavigationMenuContent,
} from "@/components/ui/navigation-menu";
import { useAuth } from "@/providers/auth-provider";

import { Search, Settings, User } from "lucide-react";
import { Input } from "@/components/ui/input";

export function NavBar() {
  const { user } = useAuth();
  return (
    <div className="flex h-12 items-center justify-between px-4 bg-card border-b border-border shadow-sm">
      <div className="flex items-center gap-6 h-full">
        <Link
          href="/"
          className="flex items-center hover:opacity-80 transition-opacity group"
        >
          <span className="font-black text-xl tracking-tighter text-primary italic">
            CASTELIA
          </span>
        </Link>
        <Link
          href="/"
          className="text-sm font-bold hover:text-primary transition-colors flex items-center h-full border-b-2 border-transparent hover:border-primary px-1"
        >
          Browse
        </Link>
      </div>

      <div className="flex-1 max-w-md px-4 hidden md:block">
        <div className="relative group">
          <Input
            placeholder="Search"
            className="h-9 w-full bg-muted/50 border-border group-focus-within:bg-background group-focus-within:ring-1 group-focus-within:ring-primary/50 transition-all pl-3 pr-10"
          />
          <button className="absolute right-0 top-0 h-full px-3 bg-muted border-l border-border rounded-r-md hover:bg-muted/80 transition-colors">
            <Search size={16} className="text-muted-foreground" />
          </button>
        </div>
      </div>

      <div className="flex items-center gap-3">
        {!user ? (
          <div className="flex items-center gap-2">
            <Link
              href="/login"
              className="text-xs font-bold bg-muted hover:bg-muted/80 transition-colors px-3 py-1.5 rounded-md"
            >
              Log In
            </Link>
            <Link
              href="/signup"
              className="text-xs font-bold bg-primary text-primary-foreground hover:bg-primary/90 transition-colors px-3 py-1.5 rounded-md"
            >
              Sign Up
            </Link>
          </div>
        ) : (
          <div className="flex items-center gap-3">
            <Link
              href="/live"
              className="text-xs font-bold text-primary hover:bg-primary/10 transition-colors px-3 py-1.5 rounded-md hidden sm:block"
            >
              Go Live
            </Link>
            <NavigationMenu viewport={false}>
              <NavigationMenuList>
                <NavigationMenuItem>
                  <NavigationMenuTrigger className="bg-transparent p-0 hover:bg-transparent focus:bg-transparent data-[state=open]:bg-transparent">
                    <div className="size-8 rounded-full bg-primary/20 border border-primary/30 flex items-center justify-center text-xs font-bold text-primary hover:scale-105 transition-transform">
                      {user.username.charAt(0).toUpperCase()}
                    </div>
                  </NavigationMenuTrigger>
                  <NavigationMenuContent className="right-0 left-auto translate-x-0">
                    <ul className="grid w-[220px] gap-1 p-2 bg-card border border-border shadow-xl">
                      <div className="px-3 py-2 border-b border-border mb-1">
                        <p className="text-sm font-bold text-primary truncate">
                          {user.username}
                        </p>
                        <p className="text-[10px] text-muted-foreground">
                          Streaming
                        </p>
                      </div>
                      <li>
                        <NavigationMenuLink asChild>
                          <Link
                            href="/channel/[channel]"
                            as={`/channel/${user.username}`}
                            className="flex w-full items-center gap-2 rounded-md p-2.5 text-xs font-bold hover:bg-muted transition-colors"
                          >
                            <User size={14} className="text-muted-foreground" />
                            Channel
                          </Link>
                        </NavigationMenuLink>
                      </li>
                      <li>
                        <NavigationMenuLink asChild>
                          <Link
                            href="/settings"
                            className="flex w-full items-center gap-2 rounded-md p-2.5 text-xs font-bold hover:bg-muted transition-colors"
                          >
                            <Settings
                              size={14}
                              className="text-muted-foreground"
                            />
                            Settings
                          </Link>
                        </NavigationMenuLink>
                      </li>
                      <div className="mt-1 pt-1 border-t border-border">
                        <button className="flex w-full items-center gap-2 rounded-md p-2.5 text-xs font-bold hover:bg-muted text-destructive transition-colors">
                          Log Out
                        </button>
                      </div>
                    </ul>
                  </NavigationMenuContent>
                </NavigationMenuItem>
              </NavigationMenuList>
            </NavigationMenu>
          </div>
        )}
        {!user && (
          <button className="p-1.5 hover:bg-muted rounded-md transition-colors">
            <User size={20} className="text-muted-foreground" />
          </button>
        )}
      </div>
    </div>
  );
}
