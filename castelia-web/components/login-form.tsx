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
      <Card className="w-96">
        <CardHeader>
          <CardTitle>Login</CardTitle>
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
            <Label htmlFor="password">Password</Label>
            <Input id="password" name="password" type="password" required />
          </div>
          {loginErr && (
            <CardDescription className="text-red-500">
              {loginErr}
            </CardDescription>
          )}
        </CardContent>
        <CardFooter>
          <Button type="submit">Login</Button>
        </CardFooter>
      </Card>
    </form>
  );
}
