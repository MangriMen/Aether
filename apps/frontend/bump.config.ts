import type { VersionBumpOptions } from 'bumpp';

import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'bumpp';
import { execa } from 'execa';

// Resolve absolute paths based on current file location
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TAURI_DIR = path.resolve(__dirname, '../../apps/desktop');
const MONOREPO_ROOT = path.resolve(__dirname, '../..');
// bumpp does not export its Operation type; take it from the execute hook.
type BumpContext = Parameters<
  Extract<VersionBumpOptions['execute'], (...args: never[]) => unknown>
>[0];

const CARGO_LOCK_PATH = path.resolve(MONOREPO_ROOT, 'Cargo.lock');
const CHANGELOG_PATH = path.resolve(MONOREPO_ROOT, 'CHANGELOG.md');

export default defineConfig({
  push: false,
  execute: async (ctx) => {
    await validateChangelog();

    const tauriFiles = await syncTauriVersion(ctx);
    await syncCargoLock();

    await updateChangelog(ctx);

    // Without `all`, bumpp commits only the files it lists. Add the ones
    // edited here so the release commit (and the tag) carry the new version.
    ctx.update({
      updatedFiles: [
        ...ctx.state.updatedFiles,
        ...tauriFiles,
        CARGO_LOCK_PATH,
        CHANGELOG_PATH,
      ],
    });
  },
});

const syncTauriVersion = async (ctx: BumpContext) => {
  const { newVersion, currentVersion } = ctx.state;

  const confPath = path.join(TAURI_DIR, 'tauri.conf.json');
  const conf = await readFile(confPath, 'utf-8');
  const updatedConf = conf.replace(
    /("version"\s*:\s*")[^"]+(")/,
    `$1${newVersion}$2`,
  );
  await writeFile(confPath, updatedConf, 'utf-8');

  const cargoPath = path.join(TAURI_DIR, 'Cargo.toml');
  const cargo = await readFile(cargoPath, 'utf-8');
  const updatedCargo = cargo.replace(
    new RegExp(`(^version\\s*=\\s*")${escapeRegex(currentVersion)}(")`, 'm'),
    `$1${newVersion}$2`,
  );
  await writeFile(cargoPath, updatedCargo, 'utf-8');

  return [confPath, cargoPath];
};

function escapeRegex(str: string) {
  return str.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, '\\$&');
}

const syncCargoLock = async () => {
  await execa('cargo', ['update', '--workspace', '--offline'], {
    cwd: MONOREPO_ROOT,
    stdio: 'inherit',
  });
};

const readChangelog = () => readFile(CHANGELOG_PATH, 'utf-8');
const writeChangelog = (data: string) => writeFile(CHANGELOG_PATH, data);

const UNRELEASED_LINK = /^\[Unreleased\]: .*$/m;
const REPO_COMPARE_URL = 'https://github.com/MangriMen/Aether/compare';

const validateChangelog = async () => {
  const content = await readChangelog();

  const unreleasedMatch = content.match(
    /## \[Unreleased\]([\s\S]*?)(?=## \[|$)/,
  );
  const unreleasedContent = unreleasedMatch?.[1]?.trim() ?? '';

  const realContent = unreleasedContent.replace(/### .+/g, '').trim();

  if (!realContent) {
    throw new Error(
      'CHANGELOG.md: The [Unreleased] section is empty! Please add some notes before bumping.',
    );
  }

  // Fail before any file is written, not halfway through the bump.
  if (!UNRELEASED_LINK.test(content)) {
    throw new Error('CHANGELOG.md: the [Unreleased] compare link is missing.');
  }
};

const updateChangelog = async (ctx: BumpContext) => {
  const { newVersion, currentVersion } = ctx.state;
  const date = new Date().toISOString().split('T')[0];
  const newVersionHeader = `## [${newVersion}] - ${date}`;

  const content = await readChangelog();
  const unreleasedHeader = '## [Unreleased]';

  const withNewSection = content.replace(
    unreleasedHeader,
    `${unreleasedHeader}\n\n${newVersionHeader}`,
  );

  // Keep the compare links at the bottom in step with the new section.
  const updatedContent = withNewSection.replace(
    UNRELEASED_LINK,
    [
      `[Unreleased]: ${REPO_COMPARE_URL}/v${newVersion}...HEAD`,
      `[${newVersion}]: ${REPO_COMPARE_URL}/v${currentVersion}...v${newVersion}`,
    ].join('\n'),
  );

  await writeChangelog(updatedContent);
};
