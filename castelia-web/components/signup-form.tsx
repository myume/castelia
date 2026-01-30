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
      <Card className="w-96">
        <CardHeader>
          <CardTitle>Sign Up</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="username">Username</Label>
            <Input
              id="username"
              name="username"
              placeholder="Your username"
              minLength={3}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              name="email"
              type="email"
              placeholder="you@example.com"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              name="password"
              type="password"
              minLength={6}
              required
            />
          </div>
          {signupError && (
            <CardDescription className="text-red-500">
              {signupError}
            </CardDescription>
          )}
        </CardContent>
        <CardFooter>
          <Button type="submit">Sign Up</Button>
        </CardFooter>
      </Card>
    </form>
  );
}
