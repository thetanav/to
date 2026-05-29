"use client";

import { useMemo, useState } from "react";

type TaskStatus = "open" | "running" | "blocked" | "done";

type Task = {
  id: number;
  title: string;
  repo: string;
  priority: "high" | "medium" | "low";
  labels: string[];
  status: TaskStatus;
  branch?: string;
  session?: string;
  depth?: number;
};

type AgentSession = {
  id: string;
  repo: string;
  task: string;
  status: "launching" | "running" | "review" | "idle";
  branch: string;
  progress: number;
};

const tasks: Task[] = [
  {
    id: 1,
    title: "Ship streaming prompt output",
    repo: "~/dev/atlas",
    priority: "high",
    labels: ["sdk", "agents"],
    status: "running",
    branch: "feature/streaming-output",
    session: "opencode-21",
  },
  {
    id: 2,
    title: "Add project search across todos",
    repo: "~/dev/atlas",
    priority: "medium",
    labels: ["search", "ux"],
    status: "open",
    depth: 1,
  },
  {
    id: 3,
    title: "Sync task state with local file storage",
    repo: "~/dev/atlas",
    priority: "high",
    labels: ["storage"],
    status: "blocked",
    branch: "feature/local-state",
  },
  {
    id: 4,
    title: "Review agent handoff prompt wording",
    repo: "~/dev/nebula",
    priority: "low",
    labels: ["prompt", "copy"],
    status: "done",
  },
  {
    id: 5,
    title: "Build multi-session OpenCode launcher",
    repo: "~/dev/nebula",
    priority: "high",
    labels: ["opencode", "launcher"],
    status: "running",
    branch: "feature/session-launcher",
    session: "opencode-34",
  },
];

const sessions: AgentSession[] = [
  {
    id: "opencode-21",
    repo: "~/dev/atlas",
    task: "Ship streaming prompt output",
    status: "running",
    branch: "feature/streaming-output",
    progress: 72,
  },
  {
    id: "opencode-34",
    repo: "~/dev/nebula",
    task: "Build multi-session OpenCode launcher",
    status: "launching",
    branch: "feature/session-launcher",
    progress: 18,
  },
  {
    id: "opencode-08",
    repo: "~/dev/atlas",
    task: "Write task state sync",
    status: "review",
    branch: "feature/local-state",
    progress: 91,
  },
];

const statusStyles: Record<TaskStatus, string> = {
  open: "border-white/10 bg-white/5 text-slate-200",
  running: "border-cyan-400/30 bg-cyan-400/10 text-cyan-200",
  blocked: "border-amber-400/30 bg-amber-400/10 text-amber-200",
  done: "border-emerald-400/30 bg-emerald-400/10 text-emerald-200",
};

const sessionStyles = {
  launching: "from-fuchsia-500/20 to-transparent",
  running: "from-cyan-500/20 to-transparent",
  review: "from-emerald-500/20 to-transparent",
  idle: "from-white/10 to-transparent",
};

export default function Home() {
  const [activeTaskId, setActiveTaskId] = useState(tasks[0].id);
  const activeTask = useMemo(
    () => tasks.find((task) => task.id === activeTaskId) ?? tasks[0],
    [activeTaskId],
  );

  const totals = useMemo(
    () => ({
      open: tasks.filter((task) => task.status === "open").length,
      running: tasks.filter((task) => task.status === "running").length,
      blocked: tasks.filter((task) => task.status === "blocked").length,
      done: tasks.filter((task) => task.status === "done").length,
    }),
    [],
  );

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(34,211,238,0.14),_transparent_30%),radial-gradient(circle_at_80%_10%,_rgba(168,85,247,0.15),_transparent_22%),linear-gradient(180deg,_#050816,_#03050d_55%,_#020307)] px-4 py-4 text-slate-100 md:px-6 md:py-6">
      <div className="mx-auto grid min-h-[calc(100vh-2rem)] max-w-[1600px] gap-4 lg:grid-cols-[280px_minmax(0,1fr)_360px]">
        <aside className="rounded-[28px] border border-white/10 bg-white/5 p-5 shadow-[0_30px_80px_rgba(0,0,0,0.45)] backdrop-blur-xl">
          <div className="flex items-center gap-3 border-b border-white/10 pb-5">
            <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-gradient-to-br from-cyan-400 to-fuchsia-500 font-mono text-sm font-semibold text-slate-950">
              to
            </div>
            <div>
              <p className="text-xs uppercase tracking-[0.35em] text-slate-400">
                control room
              </p>
              <h1 className="text-lg font-semibold">OpenCode Ops</h1>
            </div>
          </div>

          <div className="mt-5 space-y-3">
            {[
              ["All tasks", tasks.length],
              ["Open", totals.open],
              ["Running", totals.running],
              ["Blocked", totals.blocked],
              ["Done", totals.done],
            ].map(([label, value]) => (
              <button
                key={label}
                className="flex w-full items-center justify-between rounded-2xl border border-white/10 bg-black/20 px-4 py-3 text-left transition hover:border-cyan-400/30 hover:bg-white/10"
                type="button"
              >
                <span className="text-sm text-slate-300">{label}</span>
                <span className="text-sm font-semibold text-white">
                  {value}
                </span>
              </button>
            ))}
          </div>

          <div className="mt-5 rounded-3xl border border-white/10 bg-black/20 p-4">
            <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
              Quick launch
            </p>
            <div className="mt-3 space-y-3 text-sm text-slate-300">
              <div className="rounded-2xl border border-white/10 bg-slate-950/60 p-3 font-mono text-xs text-cyan-200">
                opencode --prompt {'"'}work on task 1{'"'}
              </div>
              <div className="rounded-2xl border border-white/10 bg-slate-950/60 p-3 font-mono text-xs text-cyan-200">
                opencode --prompt {'"'}resume opencode-34{'"'}
              </div>
            </div>
          </div>

          <div className="mt-5 rounded-3xl border border-white/10 bg-black/20 p-4">
            <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
              Repos
            </p>
            <div className="mt-3 space-y-2 text-sm text-slate-300">
              {Array.from(new Set(tasks.map((task) => task.repo))).map(
                (repo) => (
                  <div
                    key={repo}
                    className="flex items-center justify-between rounded-2xl border border-white/10 px-3 py-2"
                  >
                    <span>{repo}</span>
                    <span className="text-xs text-slate-500">
                      {tasks.filter((task) => task.repo === repo).length}
                    </span>
                  </div>
                ),
              )}
            </div>
          </div>
        </aside>

        <section className="rounded-[28px] border border-white/10 bg-white/5 p-5 shadow-[0_30px_80px_rgba(0,0,0,0.4)] backdrop-blur-xl">
          <div className="flex flex-col gap-5 border-b border-white/10 pb-5 lg:flex-row lg:items-end lg:justify-between">
            <div>
              <p className="text-xs uppercase tracking-[0.35em] text-slate-400">
                Developer todo cockpit
              </p>
              <h2 className="mt-2 text-3xl font-semibold tracking-tight md:text-4xl">
                Tasks, agents, and OpenCode in one place.
              </h2>
              <p className="mt-3 max-w-2xl text-sm leading-6 text-slate-300">
                See what is active, what is blocked, and what can be launched
                immediately. Designed for keyboard-first, local-first workflows.
              </p>
            </div>

            <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              {[
                ["Open", totals.open],
                ["Running", totals.running],
                ["Blocked", totals.blocked],
                ["Done", totals.done],
              ].map(([label, value]) => (
                <div
                  key={label}
                  className="rounded-2xl border border-white/10 bg-black/20 px-4 py-3"
                >
                  <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
                    {label}
                  </p>
                  <p className="mt-2 text-2xl font-semibold">{value}</p>
                </div>
              ))}
            </div>
          </div>

          <div className="mt-5 grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
            <div className="rounded-3xl border border-white/10 bg-black/20 p-4">
              <div className="flex items-center justify-between">
                <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
                  Todo graph
                </p>
                <button
                  className="rounded-full border border-cyan-400/30 bg-cyan-400/10 px-3 py-1 text-xs text-cyan-200"
                  type="button"
                >
                  + new task
                </button>
              </div>

              <div className="mt-4 space-y-3">
                {tasks.map((task) => (
                  <button
                    key={task.id}
                    type="button"
                    onClick={() => setActiveTaskId(task.id)}
                    className={`w-full rounded-3xl border px-4 py-4 text-left transition ${
                      activeTask.id === task.id
                        ? "border-cyan-400/40 bg-cyan-400/10"
                        : "border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/10"
                    }`}
                  >
                    <div className="flex flex-wrap items-start justify-between gap-3">
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-sm font-semibold text-white">
                            {task.title}
                          </span>
                          <span
                            className={`rounded-full border px-2 py-0.5 text-[11px] uppercase tracking-[0.2em] ${statusStyles[task.status]}`}
                          >
                            {task.status}
                          </span>
                        </div>
                        <p className="mt-2 text-xs text-slate-400">
                          {task.repo}
                        </p>
                      </div>

                      <div className="flex items-center gap-2 text-xs text-slate-400">
                        <span className="rounded-full border border-white/10 bg-black/30 px-2 py-1">
                          #{task.id}
                        </span>
                        <span>{task.priority}</span>
                      </div>
                    </div>

                    <div className="mt-4 flex flex-wrap gap-2">
                      {task.labels.map((label) => (
                        <span
                          key={label}
                          className="rounded-full border border-white/10 bg-black/30 px-2 py-1 text-xs text-slate-300"
                        >
                          #{label}
                        </span>
                      ))}
                      {task.branch ? (
                        <span className="rounded-full border border-fuchsia-400/20 bg-fuchsia-400/10 px-2 py-1 text-xs text-fuchsia-200">
                          {task.branch}
                        </span>
                      ) : null}
                    </div>
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-4">
              <div className="rounded-3xl border border-white/10 bg-black/20 p-4">
                <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
                  Selected task
                </p>
                <h3 className="mt-2 text-2xl font-semibold">
                  {activeTask.title}
                </h3>
                <div className="mt-4 space-y-3 text-sm text-slate-300">
                  <div className="flex items-center justify-between rounded-2xl border border-white/10 bg-white/5 px-3 py-2">
                    <span>Repo</span>
                    <span className="text-slate-100">{activeTask.repo}</span>
                  </div>
                  <div className="flex items-center justify-between rounded-2xl border border-white/10 bg-white/5 px-3 py-2">
                    <span>Priority</span>
                    <span className="text-slate-100">
                      {activeTask.priority}
                    </span>
                  </div>
                  <div className="flex items-center justify-between rounded-2xl border border-white/10 bg-white/5 px-3 py-2">
                    <span>Session</span>
                    <span className="text-slate-100">
                      {activeTask.session ?? "not launched"}
                    </span>
                  </div>
                  <div className="flex items-center justify-between rounded-2xl border border-white/10 bg-white/5 px-3 py-2">
                    <span>Branch</span>
                    <span className="text-slate-100">
                      {activeTask.branch ?? "none"}
                    </span>
                  </div>
                </div>

                <div className="mt-4 grid grid-cols-2 gap-3">
                  <button
                    className="rounded-2xl bg-gradient-to-r from-cyan-400 to-fuchsia-500 px-4 py-3 text-sm font-semibold text-slate-950"
                    type="button"
                  >
                    Open in OpenCode
                  </button>
                  <button
                    className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm font-semibold text-white"
                    type="button"
                  >
                    Mark done
                  </button>
                </div>
              </div>

              <div className="rounded-3xl border border-white/10 bg-black/20 p-4">
                <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
                  Command preview
                </p>
                <div className="mt-3 rounded-2xl border border-white/10 bg-slate-950/70 p-4 font-mono text-xs leading-5 text-cyan-200">
                  <p>
                    $ opencode --prompt {'"'}Work on task {activeTask.id}:{" "}
                    {activeTask.title}
                    {'"'}
                  </p>
                  <p>$ to do {activeTask.id} --create-branch</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <aside className="rounded-[28px] border border-white/10 bg-white/5 p-5 shadow-[0_30px_80px_rgba(0,0,0,0.4)] backdrop-blur-xl">
          <div className="flex items-center justify-between border-b border-white/10 pb-4">
            <div>
              <p className="text-xs uppercase tracking-[0.35em] text-slate-400">
                Live agents
              </p>
              <h3 className="mt-2 text-xl font-semibold">Working now</h3>
            </div>
            <span className="rounded-full border border-emerald-400/30 bg-emerald-400/10 px-3 py-1 text-xs text-emerald-200">
              3 active
            </span>
          </div>

          <div className="mt-4 space-y-4">
            {sessions.map((session) => (
              <div
                key={session.id}
                className={`rounded-3xl border border-white/10 bg-gradient-to-b p-4 ${sessionStyles[session.status]}`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
                      {session.id}
                    </p>
                    <h4 className="mt-2 font-semibold text-white">
                      {session.task}
                    </h4>
                  </div>
                  <span className="rounded-full border border-white/10 bg-black/30 px-2 py-1 text-xs text-slate-200">
                    {session.status}
                  </span>
                </div>

                <div className="mt-3 space-y-2 text-sm text-slate-300">
                  <div className="flex items-center justify-between">
                    <span>{session.repo}</span>
                    <span>{session.branch}</span>
                  </div>
                  <div className="h-2 overflow-hidden rounded-full bg-black/40">
                    <div
                      className="h-full rounded-full bg-gradient-to-r from-cyan-400 via-fuchsia-400 to-emerald-400"
                      style={{ width: `${session.progress}%` }}
                    />
                  </div>
                  <div className="flex items-center justify-between text-xs text-slate-400">
                    <span>progress</span>
                    <span>{session.progress}%</span>
                  </div>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-5 rounded-3xl border border-white/10 bg-black/20 p-4">
            <p className="text-xs uppercase tracking-[0.3em] text-slate-400">
              Activity
            </p>
            <div className="mt-3 space-y-3 text-sm text-slate-300">
              {[
                "Spawned OpenCode for task 1",
                "Linked branch feature/session-launcher",
                "Synced 12 tasks from local store",
                "Marked review task as done",
              ].map((line) => (
                <div
                  key={line}
                  className="rounded-2xl border border-white/10 bg-white/5 px-3 py-2"
                >
                  {line}
                </div>
              ))}
            </div>
          </div>
        </aside>
      </div>
    </main>
  );
}
