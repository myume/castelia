"use client";

import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

export type User = {
  id: string;
  username: string;
};

export type AuthContextType = {
  accessToken: string | null;
  login: (username: string, password: string) => Promise<void>;
  user: User | null;
};

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const auth = useContext(AuthContext);
  if (auth === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return auth;
};

export function AuthProvider({ children }: { children: ReactNode }) {
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    refreshAccessToken();
  }, []);

  useEffect(() => {
    if (!accessToken) return;

    const interval = setInterval(
      () => {
        refreshAccessToken().catch((error) => {
          console.error("Auto-refresh failed:", error);
          setAccessToken(null);
          setUser(null);
        });
      },
      14 * 60 * 1000,
    );

    return () => clearInterval(interval);
  }, [accessToken]);

  const refreshAccessToken = async () => {
    const response = await fetch("/auth/jwt/refresh", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      credentials: "include",
    });

    if (!response.ok) {
      console.log("Could not refresh token");
      return;
    }

    const data = await response.json();
    setAccessToken(data.access_token);
    if (!user) {
      getUser(data.access_token);
    }
    console.log("refreshed access token");
  };

  const login = async (username: string, password: string) => {
    const response = await fetch("/auth/login", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({ username, password }),
    });

    if (!response.ok) {
      throw new Error(await response.text());
    }

    const data = await response.json();
    setAccessToken(data.access_token);

    await getUser(data.access_token);
  };

  const getUser = async (jwt: string) => {
    const response = await fetch("/auth/jwt", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        Authorization: `Bearer ${jwt}`,
      },
    });

    if (!response.ok) {
    }

    const data = await response.json();
    const user = { id: data.sub, username: data.username };
    setUser(user);
  };

  const value = { accessToken, login, user };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
