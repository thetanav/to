import type { Metadata } from "next";
import { Instrument_Sans, Space_Grotesk, Geist } from "next/font/google";
import "./globals.css";
import { cn } from "@/lib/utils";

const display = Space_Grotesk({
  variable: "--font-display",
  subsets: ["latin"],
});

const geist = Geist({subsets:['latin'],variable:'--font-sans'});

export const metadata: Metadata = {
  title: "to - Developer Todo Control Room",
  description:
    "A local-first dashboard for developer TODOs, OpenCode sessions, and fast task handoff.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={cn("h-full", "antialiased", display.variable, "font-sans", geist.variable)}
    >
      <body className="min-h-full flex flex-col">{children}</body>
    </html>
  );
}
