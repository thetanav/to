export default function Home() {
  return (
    <div className="relative min-h-screen overflow-hidden bg-background text-foreground">
      <div className="pointer-events-none absolute inset-0 hero-ring" />
      <div className="pointer-events-none absolute inset-0 grid-fade" />
      <div className="pointer-events-none absolute inset-0 opacity-40 noise" />

      <header className="relative z-10 flex items-center justify-between px-6 pt-6 md:px-12">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-full border border-[#2a2a2a] bg-[#0b0b0b] text-sm font-semibold">
            to
          </div>
          <div>
            <p className="text-xs uppercase tracking-[0.3em] text-muted">project TODOs</p>
            <p className="font-display text-lg">to</p>
          </div>
        </div>
        <nav className="hidden items-center gap-8 text-sm text-muted md:flex">
          <a className="transition hover:text-foreground" href="#features">
            Features
          </a>
          <a className="transition hover:text-foreground" href="#workflow">
            Workflow
          </a>
          <a className="transition hover:text-foreground" href="#workflow-proof">
            Proof
          </a>
        </nav>
        <div className="flex items-center gap-3 text-sm">
          <a className="btn-secondary rounded-full px-4 py-2" href="https://github.com/thetanav/to" target="_blank" rel="noreferrer">
            GitHub
          </a>
          <a className="btn-primary rounded-full px-4 py-2 font-semibold" href="#cta">
            Install tto
          </a>
        </div>
      </header>

      <main className="relative z-10">
        <section className="px-6 pb-20 pt-16 md:px-12 md:pt-24">
          <div className="mx-auto grid max-w-6xl gap-12 lg:grid-cols-[1.1fr_0.9fr]">
            <div className="space-y-8">
              <div className="inline-flex items-center gap-3 rounded-full border border-[#2a2a2a] bg-[#0b0b0b] px-4 py-2 text-xs text-muted">
                <span className="badge rounded-full px-3 py-1 text-[11px] uppercase tracking-[0.2em]">
                  Built for velocity
                </span>
                Project-scoped TODOs for shipping teams
              </div>
              <div className="space-y-6">
                <h1 className="fade-up font-display text-4xl leading-[1.05] tracking-tight md:text-6xl">
                  A focused TODO system that lives inside every repo.
                </h1>
                <p className="fade-up-delay max-w-2xl text-lg text-muted md:text-xl">
                  to is a fast, local-first CLI that keeps TODOs scoped to the project, surfaces the next task instantly, and makes AI handoffs reliable.
                </p>
                <p className="text-xs uppercase tracking-[0.3em] text-muted">CLI command: tto</p>
              </div>
              <div className="fade-up-delay-2 flex flex-wrap items-center gap-4 text-sm">
                <a className="btn-primary rounded-full px-6 py-3 font-semibold" href="#cta">
                  Install tto
                </a>
                <a className="btn-secondary rounded-full px-6 py-3" href="#features">
                  See the workflow
                </a>
                <div className="flex items-center gap-2 text-xs text-muted">
                  <span className="h-2 w-2 rounded-full bg-[#e5e5e5]" />
                  Setup in minutes
                </div>
              </div>
              <div className="fade-up-delay-3 flex flex-wrap gap-6 text-xs uppercase tracking-[0.22em] text-muted">
                <span>Local-first</span>
                <span>Agent-ready</span>
                <span>Repo-scoped</span>
                <span>Team-safe</span>
              </div>
            </div>

            <div className="card glow relative overflow-hidden rounded-3xl p-6">
              <div className="absolute -right-10 -top-10 h-40 w-40 rounded-full bg-[rgba(255,255,255,0.12)] blur-3xl" />
              <div className="absolute bottom-6 right-6 h-24 w-24 rounded-full bg-[rgba(255,255,255,0.08)] blur-2xl" />
              <div className="relative space-y-6">
                <div className="flex items-center justify-between">
                  <p className="text-xs uppercase tracking-[0.3em] text-muted">Live snapshot</p>
                  <span className="badge rounded-full px-3 py-1 text-[11px]">CLI session</span>
                </div>
                <div className="rounded-2xl border border-[#1f1f1f] bg-[#0a0a0a] p-4 text-sm text-[#f5f5f5]">
                  <pre className="whitespace-pre-wrap font-mono text-[12px] leading-5 text-[#d4d4d4]">
{`$ tto init
Initialized /projects/api/.todo
Updated agent doc /projects/api/CLAUDE.md

$ tto add "critical API fix" --priority high --label backend
Added task 4: critical API fix @high #backend

$ tto next
Next task: 4. critical API fix

$ tto do 4 --create-branch
# opens opencode on feature/critical-api-fix`}
                  </pre>
                </div>
                <div className="grid gap-3 text-xs text-muted">
                  <div className="flex items-center justify-between">
                    <span>Local file</span>
                    <span className="text-foreground">.todo</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span>Branch created</span>
                    <span className="text-foreground">feature/critical-api-fix</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span>Open tasks</span>
                    <span className="text-foreground">4</span>
                  </div>
                </div>
                <div className="rounded-2xl border border-[#1f1f1f] bg-[#0f0f0f] p-4">
                  <div className="flex items-center justify-between text-xs text-muted">
                    <span>Agent handoff</span>
                    <span className="text-foreground">Opencode ready</span>
                  </div>
                  <div className="mt-3 h-2 w-full overflow-hidden rounded-full bg-[#141414]">
                    <div className="h-full w-4/5 rounded-full bg-gradient-to-r from-[#f5f5f5] via-[#d4d4d4] to-[#a3a3a3]" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="px-6 pb-16 md:px-12" id="features">
          <div className="mx-auto max-w-6xl">
            <div className="flex flex-wrap items-center justify-between gap-6">
              <div className="space-y-3">
                <p className="text-xs uppercase tracking-[0.4em] text-muted">Why teams ship faster</p>
                <h2 className="font-display text-3xl md:text-4xl">All tasks stay close to the code.</h2>
              </div>
                <p className="max-w-lg text-sm text-muted">
                  Keep work aligned with the repo you are in. No more stray TODOs across docs, tickets, and chat. to is the single source of truth in every project.
                </p>
            </div>

            <div className="mt-10 grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {[
                {
                  title: "Project-scoped discovery",
                  copy: "Walks up to the nearest .todo automatically so every command lands in the right repo.",
                },
                {
                  title: "Priority and labels",
                  copy: "Tag tasks with @high, @medium, and #labels for instant filtering and focus.",
                },
                {
                  title: "Agent-ready handoff",
                  copy: "tto do opens opencode with an embedded prompt and your selected tasks.",
                },
                {
                  title: "Scan codebase",
                  copy: "Imports TODO comments from git-tracked files so nothing ships unfinished.",
                },
                {
                  title: "Branch automation",
                  copy: "Create or switch to task branches automatically with --create-branch or -b.",
                },
                {
                  title: "Tree views",
                  copy: "Nested subtasks render cleanly to keep large epics visible.",
                },
              ].map((item) => (
                <div key={item.title} className="card rounded-2xl p-6">
                  <h3 className="font-display text-lg">{item.title}</h3>
                  <p className="mt-3 text-sm text-muted">{item.copy}</p>
                  <div className="mt-6 h-1 w-16 rounded-full bg-gradient-to-r from-[#f5f5f5] to-[#a3a3a3]" />
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="px-6 pb-20 md:px-12" id="workflow">
          <div className="mx-auto grid max-w-6xl gap-10 lg:grid-cols-[0.9fr_1.1fr]">
            <div className="space-y-6">
              <p className="text-xs uppercase tracking-[0.4em] text-muted">Workflow</p>
              <h2 className="font-display text-3xl md:text-4xl">
                A senior workflow that fits your team's muscle memory.
              </h2>
              <p className="text-sm text-muted">
                to keeps project TODOs close to the code. init, add, scan, and branch in minutes. No browser tabs, no context switching, just forward motion.
              </p>
              <div className="space-y-4 text-sm">
                {[
                  "Initialize once per repo with tto init.",
                  "Capture tasks with priority, labels, and nested subtasks.",
                  "Pull TODOs from code with tto scan.",
                  "Open the next task or jump into a branch with tto do.",
                ].map((step, index) => (
                  <div key={step} className="flex items-start gap-3">
                    <span className="mt-1 flex h-6 w-6 items-center justify-center rounded-full border border-[#2a2a2a] text-xs">
                      {index + 1}
                    </span>
                    <span className="text-muted">{step}</span>
                  </div>
                ))}
              </div>
            </div>
            <div className="card rounded-3xl p-6">
              <div className="flex items-center justify-between">
                <p className="text-xs uppercase tracking-[0.3em] text-muted">Commands</p>
                <span className="badge rounded-full px-3 py-1 text-[11px]">always local</span>
              </div>
              <div className="mt-6 grid gap-4 text-sm">
                {[
                  {
                    cmd: "tto ls --priority high",
                    desc: "Filter the urgent work without losing ordering.",
                  },
                  {
                    cmd: "tto add \"ship onboarding\" --label growth",
                    desc: "Quick capture with labels that sync intent.",
                  },
                  {
                    cmd: "tto tree 12",
                    desc: "See nested subtasks for large epics.",
                  },
                  {
                    cmd: "tto do 12 --create-branch",
                    desc: "Spin up the task branch instantly.",
                  },
                ].map((row) => (
                  <div key={row.cmd} className="rounded-2xl border border-[#1f1f1f] bg-[#0b0b0b] p-4">
                    <p className="font-mono text-xs text-[#e5e5e5]">{row.cmd}</p>
                    <p className="mt-2 text-xs text-muted">{row.desc}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>

        <section className="px-6 pb-20 md:px-12" id="workflow-proof">
          <div className="mx-auto max-w-6xl">
            <div className="grid gap-8 lg:grid-cols-[1.05fr_0.95fr]">
              <div className="space-y-4">
                <p className="text-xs uppercase tracking-[0.4em] text-muted">Proof points</p>
                <h2 className="font-display text-3xl md:text-4xl">
                  A simple way to keep TODOs real.
                </h2>
                <p className="text-sm text-muted">
                  Local files keep ownership clear and avoid vendor lock-in. Perfect for founders who ship fast and teams who ship together.
                </p>
              </div>
              <div className="card rounded-3xl p-6">
                <div className="grid gap-4 text-sm text-muted">
                  {[
                    {
                      title: "Zero setup overhead",
                      copy: "Initialize once and every repo knows where to look.",
                    },
                    {
                      title: "Composable by default",
                      copy: "Works with git, opencode, and any editor you already use.",
                    },
                    {
                      title: "Built for focus",
                      copy: "Instant filters keep urgent work visible without reshuffling.",
                    },
                  ].map((item) => (
                    <div key={item.title} className="rounded-2xl border border-[#1f1f1f] bg-[#0b0b0b] p-4">
                      <p className="font-display text-base text-foreground">{item.title}</p>
                      <p className="mt-2 text-xs text-muted">{item.copy}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="px-6 pb-24 md:px-12" id="cta">
          <div className="mx-auto grid max-w-6xl gap-8 lg:grid-cols-[1.05fr_0.95fr]">
            <div className="card rounded-3xl p-8">
              <p className="text-xs uppercase tracking-[0.4em] text-muted">Get started</p>
                <h2 className="mt-4 font-display text-3xl md:text-4xl">
                  Install in seconds. Ship with clarity.
                </h2>
                <p className="mt-4 text-sm text-muted">
                  Add tto to any repo and keep your work visible, prioritized, and ready for execution.
                </p>
                <div className="mt-6 rounded-2xl border border-[#1f1f1f] bg-[#0b0b0b] p-4">
                  <p className="text-xs uppercase tracking-[0.3em] text-muted">Install</p>
                  <p className="mt-2 font-mono text-sm text-[#e5e5e5]">cargo install tto</p>
                </div>
                <div className="mt-6 flex flex-wrap items-center gap-3">
                <a className="btn-primary rounded-full px-6 py-3 font-semibold" href="https://github.com/thetanav/to" target="_blank" rel="noreferrer">
                  View on GitHub
                </a>
                <a className="btn-secondary rounded-full px-6 py-3" href="https://crates.io/crates/tto" target="_blank" rel="noreferrer">
                  View on Crates
                </a>
              </div>
            </div>
            <div className="card rounded-3xl p-8" id="team">
              <p className="text-xs uppercase tracking-[0.4em] text-muted">Built for serious teams</p>
              <h3 className="mt-4 font-display text-2xl">Replace scattered TODOs with a shared standard.</h3>
              <p className="mt-3 text-sm text-muted">
                Keep tasks in git, stay portable, and give every contributor instant context. No dashboards, no tickets, just work.
              </p>
              <div className="mt-6 grid gap-3 text-sm text-muted">
                <div className="flex items-center justify-between rounded-2xl border border-[#1f1f1f] bg-[#0b0b0b] p-4">
                  <span>Works in monorepos</span>
                  <span className="text-foreground">Yes</span>
                </div>
                <div className="flex items-center justify-between rounded-2xl border border-[#1f1f1f] bg-[#0b0b0b] p-4">
                  <span>Team conventions</span>
                  <span className="text-foreground">Built-in</span>
                </div>
                <div className="flex items-center justify-between rounded-2xl border border-[#1f1f1f] bg-[#0b0b0b] p-4">
                  <span>Agent readiness</span>
                  <span className="text-foreground">Native</span>
                </div>
              </div>
              <p className="mt-4 text-xs text-muted">CLI-first workflow for teams who ship.</p>
            </div>
          </div>
        </section>

        <section className="border-t border-[#1f1f1f] px-6 py-10 md:px-12">
          <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-6 text-sm text-muted">
            <p>Built by engineers who live in the terminal.</p>
            <div className="flex items-center gap-6">
              <a className="transition hover:text-foreground" href="https://github.com/thetanav/to" target="_blank" rel="noreferrer">
                GitHub
              </a>
              <a className="transition hover:text-foreground" href="https://crates.io/crates/tto" target="_blank" rel="noreferrer">
                Crates
              </a>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
