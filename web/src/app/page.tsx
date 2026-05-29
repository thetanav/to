"use client";

import { useEffect, useMemo, useState } from "react";
import type { DashboardSnapshot, TodoTask } from "@/lib/dashboard";

const defaultDirectory = "C:/Users/Tanav Poswal/Dev/to";

function FolderIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" className="h-4 w-4" aria-hidden="true">
      <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 7.5A2.25 2.25 0 0 1 6 5.25h4.19c.6 0 1.17.24 1.59.66l1.06 1.09c.42.42 1 .66 1.59.66H18a2.25 2.25 0 0 1 2.25 2.25v7.5A2.25 2.25 0 0 1 18 19.5H6A2.25 2.25 0 0 1 3.75 17.25v-9.75Z" />
    </svg>
  );
}

function RefreshIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" className="h-4 w-4" aria-hidden="true">
      <path strokeLinecap="round" strokeLinejoin="round" d="M16.5 8.25H21v-4.5" />
      <path strokeLinecap="round" strokeLinejoin="round" d="M20.25 9a8.25 8.25 0 1 0 1.58 4.83" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" className="h-4 w-4" aria-hidden="true">
      <path strokeLinecap="round" strokeLinejoin="round" d="m9 12 2.25 2.25L15.75 9.75" />
      <path strokeLinecap="round" strokeLinejoin="round" d="M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
    </svg>
  );
}

function taskSessionStorageKey(repoRoot: string) {
  return `to.dashboard.task-sessions:${repoRoot}`;
}

function readTaskSessions(repoRoot: string): Record<number, string> {
  if (typeof window === "undefined") {
    return {};
  }

  try {
    const raw = window.localStorage.getItem(taskSessionStorageKey(repoRoot));
    if (!raw) {
      return {};
    }

    return JSON.parse(raw) as Record<number, string>;
  } catch {
    return {};
  }
}

function writeTaskSessions(repoRoot: string, sessions: Record<number, string>) {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.setItem(taskSessionStorageKey(repoRoot), JSON.stringify(sessions));
}

export default function Home() {
  const [directory, setDirectory] = useState(defaultDirectory);
  const [inputDirectory, setInputDirectory] = useState(defaultDirectory);
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedTaskId, setSelectedTaskId] = useState<number | null>(null);
  const [taskSessions, setTaskSessions] = useState<Record<number, string>>({});
  const [spawningTaskId, setSpawningTaskId] = useState<number | null>(null);
  const [spawnError, setSpawnError] = useState<string | null>(null);

  useEffect(() => {
    loadSnapshot(directory);
  }, [directory]);

  const selectedTask = useMemo(() => {
    if (!snapshot?.todo.tasks.length) {
      return null;
    }

    return snapshot.todo.tasks.find((task) => task.id === selectedTaskId) ?? snapshot.todo.tasks[0];
  }, [selectedTaskId, snapshot]);

  async function loadSnapshot(targetDirectory: string) {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/dashboard?directory=${encodeURIComponent(targetDirectory)}`);
      if (!response.ok) {
        throw new Error(`failed to load dashboard (${response.status})`);
      }

      const data = (await response.json()) as DashboardSnapshot;
      setSnapshot(data);
      setSelectedTaskId((current) => current ?? data.todo.tasks[0]?.id ?? null);
      setTaskSessions(readTaskSessions(data.repoRoot));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setSnapshot(null);
    } finally {
      setLoading(false);
    }
  }

  async function openTask(task: TodoTask) {
    setSpawningTaskId(task.id);
    setSpawnError(null);

    try {
      const response = await fetch("/api/dashboard/open", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ directory, taskId: task.id }),
      });

      if (!response.ok) {
        throw new Error("failed to open task in OpenCode");
      }

      const result = (await response.json()) as { sessionID?: string };
      if (result.sessionID) {
        const sessionID = result.sessionID;
        setTaskSessions((current) => {
          const next: Record<number, string> = { ...current, [task.id]: sessionID };
          writeTaskSessions(snapshot?.repoRoot ?? directory, next);
          return next;
        });
      }

      await loadSnapshot(directory);
    } finally {
      setSpawningTaskId(null);
    }
  }

  async function launchTask(task: TodoTask) {
    try {
      await openTask(task);
    } catch (err) {
      setSpawnError(err instanceof Error ? err.message : String(err));
    }
  }

  async function toggleTask(task: TodoTask, done: boolean) {
    const response = await fetch("/api/dashboard/toggle", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ directory, taskId: task.id, done }),
    });

    if (!response.ok) {
      throw new Error("failed to update todo state");
    }

    await loadSnapshot(directory);
  }

  const todo = snapshot?.todo;
  const opencode = snapshot?.opencode;

  return (
    <main className="min-h-screen bg-background px-4 py-4 text-foreground md:px-6 md:py-6">
      <div className="mx-auto grid min-h-[calc(100vh-2rem)] max-w-[1600px] gap-4 lg:grid-cols-[300px_minmax(0,1fr)_360px]">
        <aside className="rounded-2xl border border-border bg-card p-5 shadow-sm">
          <div className="border-b border-border pb-5">
            <h1 className="text-base font-medium">OpenCode Dashboard</h1>
          </div>

          <form
            className="mt-5 space-y-3"
            onSubmit={(event) => {
              event.preventDefault();
              setDirectory(inputDirectory);
            }}
          >
            <label className="block text-xs uppercase tracking-[0.24em] text-muted-foreground">Load repo</label>
            <div className="relative">
              <span className="pointer-events-none absolute inset-y-0 left-3 flex items-center text-muted-foreground">
                <FolderIcon />
              </span>
              <input
                value={inputDirectory}
                onChange={(event) => setInputDirectory(event.target.value)}
                className="w-full rounded-md border border-input bg-background py-2.5 pl-10 pr-3 font-mono text-xs text-foreground outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30"
                placeholder="C:/path/to/repo"
              />
            </div>
            <button className="inline-flex h-10 w-full items-center justify-center gap-2 rounded-md border border-border bg-foreground px-4 text-sm font-medium text-background transition-colors hover:bg-foreground/90" type="submit">
              <FolderIcon />
              Load repo
            </button>
          </form>

        </aside>

        <section className="rounded-2xl border border-border bg-card p-5 shadow-sm">
          <div className="flex flex-col gap-5 border-b border-border pb-5 lg:flex-row lg:items-end lg:justify-between">
            <button className="inline-flex h-10 items-center justify-center gap-2 rounded-md border border-border bg-background px-4 text-sm font-medium text-foreground transition-colors hover:bg-muted/60" type="button" onClick={() => loadSnapshot(directory)}>
              <RefreshIcon />
              Refresh
            </button>
          </div>

          <div className="mt-5 grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
            <div className="rounded-md border border-border bg-background p-4">
              <div className="flex items-center justify-between">
                <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Todo graph</p>
                <span className="rounded-full border border-border bg-muted/40 px-3 py-1 text-xs text-muted-foreground">{todo?.path ?? "no repo loaded"}</span>
              </div>

              {loading ? <p className="mt-4 text-sm text-muted-foreground">Loading repository...</p> : null}
              {error ? <p className="mt-4 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">{error}</p> : null}
              {todo?.error ? <p className="mt-4 rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-200">{todo.error}</p> : null}
              {spawnError ? <p className="mt-4 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">{spawnError}</p> : null}

              <div className="mt-4 space-y-3">
                {todo?.tasks.length ? (
                  todo.tasks.map((task) => {
                    const selected = selectedTask?.id === task.id;
                    return (
                      <div
                        key={task.id}
                        role="button"
                        tabIndex={0}
                        onClick={() => setSelectedTaskId(task.id)}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            event.preventDefault();
                            setSelectedTaskId(task.id);
                          }
                        }}
                        className={`w-full rounded-md border px-4 py-4 text-left transition-colors ${selected ? "border-foreground/30 bg-muted/60" : "border-border bg-background hover:bg-muted/40"}`}
                      >
                        <div className="flex flex-wrap items-start justify-between gap-3">
                          <div>
                            <div className="flex flex-wrap items-center gap-2">
                              <span className="text-sm font-medium text-foreground">{task.text}</span>
                              <span className={`rounded-full border px-2 py-0.5 text-[11px] uppercase tracking-[0.18em] ${task.done ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-300" : "border-border bg-muted/50 text-muted-foreground"}`}>
                                {task.done ? "done" : "open"}
                              </span>
                            </div>
                            <p className="mt-2 text-xs text-muted-foreground">{task.indent > 0 ? "subtask" : "root task"}</p>
                          </div>
                          <span className="rounded-full border border-border bg-muted/40 px-2 py-1 text-xs text-muted-foreground">#{task.id}</span>
                        </div>

                        <div className="mt-4 flex flex-wrap gap-2 text-xs">
                          {task.priority ? <span className="rounded-full border border-border bg-muted/40 px-2 py-1 text-muted-foreground">@{task.priority}</span> : null}
                          {task.labels.map((label) => (
                            <span key={label} className="rounded-full border border-border bg-muted/40 px-2 py-1 text-muted-foreground">#{label}</span>
                          ))}
                        </div>

                        <div className="mt-4 flex flex-wrap items-center gap-2">
                          <button
                            className="inline-flex h-9 items-center justify-center gap-2 rounded-md border border-border bg-background px-3 text-xs font-medium text-foreground transition-colors hover:bg-muted/60"
                            type="button"
                            onClick={async (event) => {
                              event.stopPropagation();
                              await launchTask(task);
                            }}
                            disabled={spawningTaskId === task.id}
                          >
                            {spawningTaskId === task.id ? "Running..." : "Do"}
                          </button>
                        </div>
                      </div>
                    );
                  })
                  ) : (
                    <div className="rounded-md border border-dashed border-border bg-muted/20 p-8 text-sm text-muted-foreground">
                      No `.todo` file found yet. Load a repo with `to init` run in it or create one in the project root.
                    </div>
                  )}
              </div>
            </div>

            <div className="rounded-md border border-border bg-background p-4">
              <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Selected task</p>
              {selectedTask ? (
                <>
                  <h3 className="mt-2 text-2xl font-semibold tracking-tight">{selectedTask.text}</h3>
                  <button className="mt-4 inline-flex h-10 items-center justify-center gap-2 rounded-md border border-border bg-background px-4 text-sm font-medium text-foreground transition-colors hover:bg-muted/60" type="button" onClick={() => toggleTask(selectedTask, !selectedTask.done)}>
                    <CheckIcon />
                    Mark {selectedTask.done ? "open" : "done"}
                  </button>
                </>
              ) : null}
            </div>
          </div>
        </section>

        <aside className="rounded-2xl border border-border bg-card p-5 shadow-sm">
          <div className="flex items-center justify-between border-b border-border pb-4">
            <div>
              <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Workspace</p>
              <h3 className="mt-2 text-xl font-semibold">Live state</h3>
            </div>
            <span className="rounded-full border border-border bg-muted/40 px-3 py-1 text-xs text-muted-foreground">
              {opencode?.connected ? "online" : "offline"}
            </span>
          </div>

          <div className="mt-4 space-y-4 text-sm text-muted-foreground">
            <div className="rounded-md border border-border bg-muted/30 p-4">
              <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Current project</p>
              <p className="mt-3 text-foreground">{opencode?.currentProject?.worktree ?? snapshot?.repoRoot ?? "No project loaded"}</p>
            </div>

            <div className="rounded-md border border-border bg-muted/30 p-4">
              <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Projects seen by OpenCode</p>
              <div className="mt-3 space-y-2">
                {opencode?.projects.length ? opencode.projects.map((project) => <div key={project.id} className="rounded-md border border-border bg-background px-3 py-2 text-xs text-muted-foreground">{project.worktree}</div>) : <p className="text-sm text-muted-foreground">No projects yet.</p>}
              </div>
            </div>

            <div className="rounded-md border border-border bg-muted/30 p-4">
              <p className="text-xs uppercase tracking-[0.24em] text-muted-foreground">Status</p>
              <p className="mt-3 text-sm text-foreground">{opencode?.error ?? "Connected and ready"}</p>
            </div>
          </div>
        </aside>
      </div>
    </main>
  );
}
