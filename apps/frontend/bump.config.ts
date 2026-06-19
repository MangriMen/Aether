import type { VersionBumpResults } from 'bumpp';

import { defineConfig } from 'bumpp';
import { execa } from 'execa';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Resolve absolute paths based on current file location
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TAURI_DIR = path.resolve(__dirname, '../../apps/desktop');
const MONOREPO_ROOT = path.resolve(__dirname, '../..');
const CHANGELOG_PATH = path.resolve(__dirname, './CHANGELOG.md');

export default defineConfig({
  all: true,
  push: false,
  execute: async (ctx) => {
    await validateChangelog();

    await syncTauriVersion(ctx);
    await syncCargoLock();

    await updateChangelog(ctx.results);
  },
});

const syncTauriVersion = async (ctx: {
  options: { cwd: string };
  state: { newVersion: string; currentVersion: string };
}) => {
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

const validateChangelog = async () => {
  const content = await readChangelog();

  const unreleasedMatch = content.match(
    /## \[Unreleased\]([\s\S]*?)(?=## \[|$)/,
  );
  const unreleasedContent = unreleasedMatch ? unreleasedMatch[1].trim() : '';

  const realContent = unreleasedContent.replace(/### .+/g, '').trim();

  if (!realContent) {
    throw new Error(
      'CHANGELOG.md: The [Unreleased] section is empty! Please add some notes before bumping.',
    );
  }
};

const updateChangelog = async (results: VersionBumpResults) => {
  const date = new Date().toISOString().split('T')[0];
  const newVersionHeader = `## [${results.newVersion}] - ${date}`;

  const content = await readChangelog();
  const unreleasedHeader = '## [Unreleased]';

  const updatedContent = content.replace(
    unreleasedHeader,
    `${unreleasedHeader}\n\n${newVersionHeader}`,
  );

  await writeChangelog(updatedContent);
};
