import { NextResponse } from "next/server";
import { findTask, openTaskInOpencode } from "@/lib/dashboard";

export async function POST(request: Request) {
  const body = (await request.json()) as { directory?: string; taskId?: number };

  if (!body.directory || !body.taskId) {
    return NextResponse.json({ error: "directory and taskId are required" }, { status: 400 });
  }

  const task = await findTask(body.directory, body.taskId);
  const result = await openTaskInOpencode(body.directory, task);
  return NextResponse.json(result);
}
