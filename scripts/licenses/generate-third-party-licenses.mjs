/**
 * Generates apps/desktop/licenses/THIRD_PARTY_LICENSES.md, which is bundled
 * into the installers (see `bundle.resources` in apps/desktop/tauri.conf.json).
 *
 * Requires `cargo-about` (`cargo install cargo-about --locked --features cli`)
 * and installed frontend dependencies (`bun install`).
 *
 * Usage:
 *   node scripts/licenses/generate-third-party-licenses.mjs
 *   bun run licenses:generate
 */
import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const outFile = join(
  root,
  "apps",
  "desktop",
  "licenses",
  "THIRD_PARTY_LICENSES.md",
);

// ── Rust dependencies (cargo-about) ──────────────────────────────────────
const rustFile = join(tmpdir(), `aether-rust-licenses-${process.pid}.md`);
execFileSync(
  "cargo",
  [
    "about",
    "generate",
    "--config",
    join(root, "scripts", "licenses", "about.toml"),
    "--output-file",
    rustFile,
    join(root, "scripts", "licenses", "about.hbs"),
  ],
  { cwd: join(root, "apps", "desktop"), stdio: "inherit" },
);
const rustSection = readFileSync(rustFile, "utf-8");

// ── Frontend production dependencies (license-checker-rseidelsohn) ───────
const frontendDir = join(root, "apps", "frontend");
const checkerOutput = execFileSync(
  "bun",
  ["x", "license-checker-rseidelsohn", "--production", "--json"],
  { cwd: frontendDir, encoding: "utf-8", maxBuffer: 256 * 1024 * 1024 },
);
const packages = JSON.parse(checkerOutput);

// `--production` lists only `dependencies`, but some devDependencies end up in
// the bundle at build time (icons are inlined by unplugin-icons, Tailwind
// preflight and plugin CSS are compiled into the stylesheet). List them too.
// `licenseFallback` is used for packages that ship no license file.
const BUNDLED_AT_BUILD = [
  {
    name: "@iconify-json/mdi",
    licenseFallback: join(
      root,
      "packages",
      "aether-core-plugin-api",
      "LICENSE-APACHE",
    ),
  },
  { name: "tailwindcss" },
  { name: "tailwindcss-animate" },
  { name: "@tailwindcss/typography" },
];

const resolvePackageDir = (name) =>
  [
    join(frontendDir, "node_modules", name),
    join(root, "node_modules", name),
  ].find((dir) => existsSync(join(dir, "package.json")));

for (const { name, licenseFallback } of BUNDLED_AT_BUILD) {
  const dir = resolvePackageDir(name);
  if (!dir) {
    throw new Error(`${name} is bundled at build time but is not installed`);
  }
  const pkg = JSON.parse(readFileSync(join(dir, "package.json"), "utf-8"));
  const info = JSON.parse(
    existsSync(join(dir, "info.json"))
      ? readFileSync(join(dir, "info.json"), "utf-8")
      : "{}",
  );
  const licenseFile = readdirSync(dir).find((f) => /^licen[sc]e/i.test(f));
  packages[`${name}@${pkg.version}`] = {
    licenses: pkg.license ?? info.license?.spdx,
    publisher: info.author?.name,
    licenseFile: licenseFile ? join(dir, licenseFile) : licenseFallback,
  };
}

// Group packages by license text so each distinct text is printed once.
const byText = new Map();
for (const [id, info] of Object.entries(packages)) {
  if (id.startsWith("@aether/")) continue;

  let text = "";
  if (info.licenseFile && !/readme/i.test(info.licenseFile)) {
    text = readFileSync(info.licenseFile, "utf-8").trim();
  }
  const license = String(info.licenses);
  if (!text)
    text = `License: ${license} (no license file shipped by the package)`;

  const entry = byText.get(text) ?? { licenses: new Set(), users: [] };
  entry.licenses.add(license);
  entry.users.push(info.publisher ? `${id} (${info.publisher})` : id);
  byText.set(text, entry);
}

let frontendSection = "# Frontend dependencies\n\n";
frontendSection +=
  "Each distinct license text is listed once, followed by the packages that use it.\n";
for (const [text, { licenses, users }] of byText) {
  frontendSection += `\n---\n\n### ${[...licenses].join(", ")}\n\nUsed by:\n`;
  frontendSection += users
    .sort()
    .map((u) => `- ${u}`)
    .join("\n");
  frontendSection += `\n\n\`\`\`text\n${text}\n\`\`\`\n`;
}

// ── Write result ─────────────────────────────────────────────────────────
const header =
  "# Third-party licenses\n\nAether bundles the open-source components listed below.\n\n";
mkdirSync(dirname(outFile), { recursive: true });
writeFileSync(outFile, `${header}${rustSection}\n${frontendSection}`);
console.log(`Wrote ${outFile}`);
