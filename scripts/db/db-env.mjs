/**
 * Set up DATABASE_URL for the current shell session.
 *
 * Usage:
 *   eval "$(node scripts/db/db-env.mjs)"
 *   eval "$(bun run scripts/db/db-env.mjs)"
 */
import { existsSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const root = resolve(dirname(__filename), "..", "..");

const envPath = resolve(root, ".env");
if (existsSync(envPath)) {
  const content = readFileSync(envPath, "utf-8");
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("DATABASE_URL=") && !trimmed.startsWith("#")) {
      const value = trimmed.slice("DATABASE_URL=".length).replace(/^["']|["']$/g, "");
      process.stdout.write(`DATABASE_URL=${value}\n`);
      process.stdout.write(`export DATABASE_URL\n`);
      process.exit(0);
    }
  }
}

process.stdout.write("# DATABASE_URL not found in .env\n");
process.exit(1);
