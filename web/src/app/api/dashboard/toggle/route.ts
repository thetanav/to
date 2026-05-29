import { NextResponse } from "next/server";
import { toggleTodoTask } from "@/lib/dashboard";

export async function POST(request: Request) {
  const body = (await request.json()) as { directory?: string; taskId?: number; done?: boolean };

  if (!body.directory || !body.taskId || typeof body.done !== "boolean") {
    return NextResponse.json({ error: "directory, taskId, and done are required" }, { status: 400 });
  }

  await toggleTodoTask(body.directory, body.taskId, body.done);
  return NextResponse.json({ ok: true });
}
