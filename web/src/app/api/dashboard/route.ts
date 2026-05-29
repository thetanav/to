import { NextResponse } from "next/server";
import { loadDashboardSnapshot } from "@/lib/dashboard";

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const directory = searchParams.get("directory") ?? process.cwd();
  const snapshot = await loadDashboardSnapshot(directory);

  return NextResponse.json(snapshot);
}
