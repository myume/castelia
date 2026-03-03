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

export function SignupForm() {
  const [signupError, setSignupError] = useState("");
  const { login } = useAuth();

  const signup = async (formData: FormData) => {
    const username = formData.get("username");
    const email = formData.get("email");
    const password = formData.get("password");
    if (!username || !email || !password) {
      setSignupError("Username, email, and password are required.");
      return;
    }
    const response = await fetch("/auth/signup", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({ username, email, password }),
    });

    if (!response.ok) {
      setSignupError(`Failed to signup: ${await response.text()}`);
      return;
    }
    try {
      await login(username.toString(), password.toString());
    } catch (e) {
      console.error(e);
      return;
    }
    redirect("/");
  };

  return (
    <form action={signup}>
      <Card className="w-96 border-muted/60 shadow-xl bg-card/50 backdrop-blur-sm">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl font-bold tracking-tight">
            Sign Up
          </CardTitle>
          <CardDescription className="text-xs font-medium">
            Create a new account to start streaming
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
              htmlFor="email"
              className="text-xs font-bold uppercase tracking-wider text-muted-foreground"
            >
              Email
            </Label>
            <Input
              id="email"
              name="email"
              type="email"
              placeholder="you@example.com"
              className="bg-muted/30 border-muted focus-visible:ring-primary/30 h-10"
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
              minLength={6}
              required
            />
          </div>
          {signupError && (
            <div className="text-destructive text-xs font-semibold bg-destructive/10 p-2 rounded border border-destructive/20">
              {signupError}
            </div>
          )}
        </CardContent>
        <CardFooter className="pt-2">
          <Button type="submit" className="w-full font-bold h-10">
            Sign Up
          </Button>
        </CardFooter>
      </Card>
    </form>
  );
}
