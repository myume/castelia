import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { NavBar } from "@/components/navbar";
import { AuthProvider } from "@/providers/auth-provider";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Castelia",
  description: "Self-hosted broadcasting",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased min-h-screen bg-background text-foreground dark`}
      >
        <AuthProvider>
          <div className="relative flex min-h-screen flex-col overflow-x-hidden">
            <header className="sticky top-0 z-50 w-full bg-card border-b border-border shadow-sm">
              <NavBar />
            </header>
            <main className="flex-1 bg-background">{children}</main>
          </div>
        </AuthProvider>
      </body>
    </html>
  );
}
