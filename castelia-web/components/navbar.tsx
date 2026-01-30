"use client";

import Link from "next/link";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu";
import { useAuth } from "@/providers/auth-provider";

export function NavBar() {
  const { user } = useAuth();
  return (
    <NavigationMenu>
      <NavigationMenuList className="flex justify-between w-screen p-1">
        <div>
          <NavigationMenuItem>
            <NavigationMenuLink
              asChild
              className={navigationMenuTriggerStyle()}
            >
              <Link href="/">Browse</Link>
            </NavigationMenuLink>
          </NavigationMenuItem>
        </div>
        {!user ? (
          <div className="flex gap-1">
            <NavigationMenuItem>
              <NavigationMenuLink
                asChild
                className={navigationMenuTriggerStyle()}
              >
                <Link href="/login">Login</Link>
              </NavigationMenuLink>
            </NavigationMenuItem>
            <NavigationMenuItem>
              <NavigationMenuLink
                asChild
                className={navigationMenuTriggerStyle()}
              >
                <Link href="/signup">Sign Up</Link>
              </NavigationMenuLink>
            </NavigationMenuItem>
          </div>
        ) : (
          <div>
            <NavigationMenuItem>
              <NavigationMenuTrigger>{user.username}</NavigationMenuTrigger>
            </NavigationMenuItem>
          </div>
        )}
      </NavigationMenuList>
    </NavigationMenu>
  );
}
