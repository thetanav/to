import { createOpencodeClient, type Project as OpenCodeProject, type Session as OpenCodeSession, type SessionStatus as OpenCodeSessionStatus } from "@opencode-ai/sdk/v2";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";

export type TodoPriority = "high" | "medium" | "low";

export type TodoTask = {
  id: number;
  lineIndex: number;
  done: boolean;
  text: string;
  indent: number;
  priority?: TodoPriority;
  labels: string[];
};

export type TodoSnapshot = {
  exists: boolean;
  path: string;
  tasks: TodoTask[];
  total: number;
  open: number;
  done: number;
  error?: string;
};

export type OpenCodeSnapshot = {
  connected: boolean;
  baseUrl: string;
  version?: string;
  error?: string;
  currentProject?: OpenCodeProject | null;
  projects: OpenCodeProject[];
  sessions: OpenCodeSession[];
  sessionStatus: Record<string, OpenCodeSessionStatus>;
  fileStatus: unknown[];
  vcsStatus: unknown;
};

export type DashboardSnapshot = {
  directory: string;
  repoRoot: string;
  todo: TodoSnapshot;
  opencode: OpenCodeSnapshot;
};

const todoLinePattern = /^(\s*)\[( |x|X)\]\s+(.*)$/;

export async function loadDashboardSnapshot(directory: string): Promise<DashboardSnapshot> {
  const repoRoot = await resolveRepoRoot(directory);
  const todoPath = path.join(repoRoot, ".todo");
  const [todo, opencode] = await Promise.all([loadTodoSnapshot(todoPath), loadOpenCodeSnapshot(repoRoot)]);

  return {
    directory: path.resolve(directory),
    repoRoot,
    todo,
    opencode,
  };
}

export async function loadTodoSnapshot(todoPath: string): Promise<TodoSnapshot> {
  try {
    const contents = await readFile(todoPath, "utf8");
    const tasks = parseTodoContents(contents);

    return {
      exists: true,
      path: todoPath,
      tasks,
      total: tasks.length,
      open: tasks.filter((task) => !task.done).length,
      done: tasks.filter((task) => task.done).length,
    };
  } catch (error) {
    if (isMissingFile(error)) {
      return {
        exists: false,
        path: todoPath,
        tasks: [],
        total: 0,
        open: 0,
        done: 0,
      };
    }

    return {
      exists: true,
      path: todoPath,
      tasks: [],
      total: 0,
      open: 0,
      done: 0,
      error: toMessage(error),
    };
  }
}

export async function toggleTodoTask(directory: string, taskId: number, done: boolean): Promise<void> {
  const repoRoot = await resolveRepoRoot(directory);
  const todoPath = path.join(repoRoot, ".todo");
  const contents = await readFile(todoPath, "utf8");
  const newline = contents.includes("\r\n") ? "\r\n" : "\n";
  const lines = contents.split(/\r?\n/);

  let count = 0;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const match = line.match(todoLinePattern);
    if (!match) {
      continue;
    }

    count += 1;
    if (count !== taskId) {
      continue;
    }

    const [, leading, , text] = match;
    lines[index] = `${leading}[${done ? "x" : " "}] ${text}`;
    await writeFile(todoPath, lines.join(newline), "utf8");
    return;
  }

  throw new Error(`task ${taskId} was not found in ${todoPath}`);
}

export async function findTask(directory: string, taskId: number): Promise<TodoTask> {
  const todo = await loadTodoSnapshot(path.join(await resolveRepoRoot(directory), ".todo"));
  const task = todo.tasks.find((item) => item.id === taskId);
  if (!task) {
    throw new Error(`task ${taskId} was not found`);
  }

  return task;
}

export async function loadOpenCodeSnapshot(directory: string): Promise<OpenCodeSnapshot> {
  const baseUrl = process.env.OPENCODE_BASE_URL ?? "http://127.0.0.1:4096";
  const client = createOpencodeClient({
    baseUrl,
    responseStyle: "data",
    throwOnError: true,
    directory,
  });

  try {
    const [currentProject, projects, sessions, sessionStatus, fileStatus, vcsStatus] = await Promise.all([
      safeCall(() => client.project.current()),
      safeCall(() => client.project.list()),
      safeCall(() => client.session.list({ limit: 20 })),
      safeCall(() => client.session.status()),
      safeCall(() => client.file.status()),
      safeCall(() => client.vcs.status()),
    ]);

    return {
      connected: true,
      baseUrl,
      currentProject: currentProject?.data ?? null,
      projects: Array.isArray(projects?.data) ? projects.data : [],
      sessions: Array.isArray(sessions?.data) ? sessions.data : [],
      sessionStatus: (sessionStatus?.data ?? {}) as Record<string, OpenCodeSessionStatus>,
      fileStatus: Array.isArray(fileStatus?.data) ? fileStatus.data : [],
      vcsStatus: vcsStatus?.data ?? null,
    };
  } catch (error) {
    return {
      connected: false,
      baseUrl,
      error: toMessage(error),
      projects: [],
      sessions: [],
      sessionStatus: {},
      fileStatus: [],
      vcsStatus: null,
    };
  }
}

export async function openTaskInOpencode(directory: string, task: TodoTask): Promise<{ sessionID: string }> {
  const repoRoot = await resolveRepoRoot(directory);
  const baseUrl = process.env.OPENCODE_BASE_URL ?? "http://127.0.0.1:4096";
  const client = createOpencodeClient({ baseUrl, responseStyle: "data", throwOnError: true, directory: repoRoot });
  const knownSessionIDs = new Set((await safeCall(() => client.session.list({ limit: 100 })))?.data?.map((session) => session.id) ?? []);

  await launchTodoCommand(repoRoot, [task.id]);

  const session = await waitForSpawnedSession(client, repoRoot, task, knownSessionIDs);
  return { sessionID: session.id };
}

async function launchTodoCommand(repoRoot: string, indices: number[]): Promise<void> {
  const workspaceRoot = resolveWorkspaceRoot();
  const command = resolveTodoCommand(workspaceRoot);
  const args = command === "cargo" ? ["run", "--quiet", "--manifest-path", path.join(workspaceRoot, "Cargo.toml"), "--", "do", ...indices.map(String)] : ["do", ...indices.map(String)];

  await new Promise<void>((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: repoRoot,
      detached: true,
      stdio: "ignore",
      windowsHide: true,
    });

    child.once("error", reject);
    child.once("spawn", () => {
      child.unref();
      resolve();
    });
  });
}

async function waitForSpawnedSession(
  client: ReturnType<typeof createOpencodeClient>,
  repoRoot: string,
  task: TodoTask,
  knownSessionIDs: Set<string>,
): Promise<OpenCodeSession> {
  const deadline = Date.now() + 15000;
  const title = `Task ${task.id}: ${task.text}`;

  while (Date.now() < deadline) {
    const result = await safeCall(() => client.session.list({ limit: 100 }));
    const sessions = Array.isArray(result?.data) ? result.data : [];
    const match = sessions
      .filter((session) => !knownSessionIDs.has(session.id) && session.directory === repoRoot && session.title === title)
      .sort((left, right) => right.time.created - left.time.created)[0];

    if (match) {
      return match;
    }

    await delay(500);
  }

  throw new Error("OpenCode session did not appear after launching `to do`");
}

function resolveWorkspaceRoot(): string {
  const cwd = process.cwd();
  const directCargo = path.join(cwd, "Cargo.toml");
  if (existsSync(directCargo)) {
    return cwd;
  }

  const parent = path.resolve(cwd, "..");
  const parentCargo = path.join(parent, "Cargo.toml");
  if (existsSync(parentCargo)) {
    return parent;
  }

  return cwd;
}

function resolveTodoCommand(workspaceRoot: string): string {
  const binaryName = process.platform === "win32" ? "to.exe" : "to";
  const binaryPath = process.env.TO_BIN ?? path.join(workspaceRoot, "target", "debug", binaryName);
  return existsSync(binaryPath) ? binaryPath : "cargo";
}


export async function resolveRepoRoot(inputDirectory: string): Promise<string> {
  let current = path.resolve(inputDirectory);

  while (true) {
    const todoPath = path.join(current, ".todo");
    if (await pathExists(todoPath)) {
      return current;
    }

    const parent = path.dirname(current);
    if (parent === current) {
      return path.resolve(inputDirectory);
    }

    current = parent;
  }
}

export function parseTodoContents(contents: string): TodoTask[] {
  const tasks: TodoTask[] = [];

  for (const [lineIndex, rawLine] of contents.split(/\r?\n/).entries()) {
    const trimmedEnd = rawLine.trimEnd();
    if (!trimmedEnd.trim()) {
      continue;
    }

    const leadingSpaces = trimmedEnd.length - trimmedEnd.trimStart().length;
    if (leadingSpaces % 2 !== 0) {
      throw new Error(`Malformed .todo line ${lineIndex + 1}: ${trimmedEnd}`);
    }

    const match = trimmedEnd.match(todoLinePattern);
    if (!match) {
      throw new Error(`Malformed .todo line ${lineIndex + 1}: ${trimmedEnd}`);
    }

    const [, , doneMarker, text] = match;
    const { title, priority, labels } = parseTaskMetadata(text.trim());

    if (!title) {
      throw new Error(`Malformed .todo line ${lineIndex + 1}: ${trimmedEnd}`);
    }

    tasks.push({
      id: tasks.length + 1,
      lineIndex,
      done: doneMarker !== " ",
      text: title,
      indent: leadingSpaces / 2,
      priority,
      labels,
    });
  }

  return tasks;
}

function parseTaskMetadata(text: string): { title: string; priority?: TodoPriority; labels: string[] } {
  const words = text.split(/\s+/).filter(Boolean);
  let priority: TodoPriority | undefined;
  const labels: string[] = [];

  while (words.length > 0) {
    const last = words[words.length - 1];
    if (last.startsWith("#")) {
      labels.push(cleanLabel(last.slice(1)));
      words.pop();
      continue;
    }

    if (last.startsWith("@")) {
      if (priority) {
        throw new Error("task can only have one priority tag");
      }

      priority = parsePriority(last.slice(1));
      words.pop();
      continue;
    }

    break;
  }

  labels.reverse();
  return { title: words.join(" "), priority, labels };
}

function parsePriority(value: string): TodoPriority {
  const lower = value.toLowerCase();
  if (lower === "high" || lower === "h") return "high";
  if (lower === "medium" || lower === "med" || lower === "m") return "medium";
  if (lower === "low" || lower === "l") return "low";

  throw new Error(`priority must be high, medium, or low; got ${value}`);
}

function cleanLabel(label: string): string {
  const value = label.trim().replace(/^#/, "");
  if (!value) {
    throw new Error("label cannot be empty");
  }

  if (!/^[a-zA-Z0-9_-]+$/.test(value)) {
    throw new Error(`label ${value} can only contain letters, numbers, hyphen, or underscore`);
  }

  return value.toLowerCase();
}

async function safeCall<T>(fn: () => Promise<T>): Promise<T | null> {
  try {
    return await fn();
  } catch {
    return null;
  }
}

async function delay(milliseconds: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function pathExists(filePath: string): Promise<boolean> {
  try {
    await readFile(filePath, "utf8");
    return true;
  } catch (error) {
    return isMissingFile(error) ? false : false;
  }
}

function isMissingFile(error: unknown): boolean {
  return typeof error === "object" && error !== null && "code" in error && (error as { code?: string }).code === "ENOENT";
}

function toMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}
