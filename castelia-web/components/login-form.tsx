"use client";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/providers/auth-provider";
import { redirect } from "next/navigation";
import { useState } from "react";

export function LoginForm() {
  const [loginErr, setLoginErr] = useState("");
  const { login } = useAuth();

  const handleSubmit = async (formData: FormData) => {
    setLoginErr("");
    const username = formData.get("username");
    const password = formData.get("password");
    if (!username || !password) {
      setLoginErr("Username and Password are required");
      return;
    }
    try {
      await login(username.toString(), password.toString());
      console.log("Logged in.");
    } catch (e: unknown) {
      if (e instanceof String) {
        setLoginErr(e.toString());
      } else {
        console.error("Failed to log in", e);
        setLoginErr("Failed to log in");
      }
      return;
    }
    redirect("/");
  };

  return (
    <form action={handleSubmit}>
      <Card className="w-96 border-muted/60 shadow-xl bg-card/50 backdrop-blur-sm">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl font-bold tracking-tight">
            Login
          </CardTitle>
          <CardDescription className="text-xs font-medium">
            Enter your credentials to access your account
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 pt-2">
          <div className="space-y-2">
            <Label
              htmlFor="username"
              className="text-xs font-bold uppercase tracking-wider text-muted-foreground"
            >
              Username
            </Label>
            <Input
              id="username"
              name="username"
              placeholder="Your username"
              className="bg-muted/30 border-muted focus-visible:ring-primary/30 h-10"
              minLength={3}
              required
            />
          </div>
          <div className="space-y-2">
            <Label
              htmlFor="password"
              className="text-xs font-bold uppercase tracking-wider text-muted-foreground"
            >
              Password
            </Label>
            <Input
              id="password"
              name="password"
              type="password"
              className="bg-muted/30 border-muted focus-visible:ring-primary/30 h-10"
              required
            />
          </div>
          {loginErr && (
            <div className="text-destructive text-xs font-semibold bg-destructive/10 p-2 rounded border border-destructive/20">
              {loginErr}
            </div>
          )}
        </CardContent>
        <CardFooter className="pt-2">
          <Button type="submit" className="w-full font-bold h-10">
            Login
          </Button>
        </CardFooter>
      </Card>
    </form>
  );
}
