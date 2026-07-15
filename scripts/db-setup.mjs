/**
 * Database setup for aether development.
 *
 * Reads `DATABASE_URL` from the workspace `.env`, creates a fresh database,
 * runs both groups of migrations (core + desktop), and prepares offline metadata.
 *
 * Usage:
 *   node scripts/db-setup.mjs
 *   bun run db:setup
 */
import { execSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, unlinkSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const root = resolve(dirname(__filename), "..");

// ── Parse DATABASE_URL from workspace root .env ──────────────────────────
const envPath = resolve(root, ".env");
let databaseUrl = "";

if (existsSync(envPath)) {
  const content = readFileSync(envPath, "utf-8");
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("DATABASE_URL=")) {
      databaseUrl = trimmed.slice("DATABASE_URL=".length).replace(/^["']|["']$/g, "");
      break;
    }
  }
}

if (!databaseUrl) {
  console.error("❌ DATABASE_URL not found in .env");
  process.exit(1);
}

// Resolve relative path (sqlite:///... for absolute, sqlite://... for relative)
const dbRelPath = databaseUrl.replace(/^sqlite:\/\//, "");
const dbPath = resolve(root, dbRelPath);
const dbDir = dirname(dbPath);

// ── Helpers ──────────────────────────────────────────────────────────────
function run(cmd, opts = {}) {
  console.log(`\n> ${cmd}`);
  execSync(cmd, { stdio: "inherit", cwd: opts.cwd ?? root, env: { ...process.env, ...opts.env }, ...opts });
}

function setEnv(name, value) {
  process.env[name] = value;
}

// ── Main ─────────────────────────────────────────────────────────────────
function main() {
  console.log("─".repeat(50));
  console.log("  Aether — Database Setup");
  console.log(`  DB path: ${dbPath}`);
  console.log("─".repeat(50));

  // 1. Remove old DB file
  if (existsSync(dbPath)) {
    unlinkSync(dbPath);
    console.log("\n🗑️  Removed old database");
  }

  // 2. Ensure DB directory exists
  mkdirSync(dbDir, { recursive: true });

  // 3. Create fresh database
  // Use absolute URL so sqlx-cli finds the right file regardless of cwd
  const absoluteDbUrl = `sqlite:///${dbPath.replace(/\\/g, "/")}`;
  setEnv("DATABASE_URL", absoluteDbUrl);
  run("sqlx db create");

  // 4. Run migrations for aether-core (uses custom table _sqlx_migrations_aether_core)
  const corePath = resolve(root, "packages/core/aether-core");
  run("sqlx migrate run", { cwd: corePath, env: { DATABASE_URL: absoluteDbUrl } });

  // 5. Run migrations for desktop app (uses default table _sqlx_migrations)
  const desktopPath = resolve(root, "apps/desktop");
  run("sqlx migrate run", { cwd: desktopPath, env: { DATABASE_URL: absoluteDbUrl } });

  // 6. Prepare offline metadata for both packages
  console.log("\n── Preparing SQLx offline metadata ──");

  run("cargo sqlx prepare -- --all-targets", { cwd: corePath, env: { DATABASE_URL: absoluteDbUrl } });
  run("cargo sqlx prepare -- --all-targets", { cwd: desktopPath, env: { DATABASE_URL: absoluteDbUrl } });

  console.log("\n✅ Database setup complete!");
}

main();
