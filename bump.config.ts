import type { VersionBumpResults } from 'bumpp';

import { defineConfig } from 'bumpp';
import { execa } from 'execa';
import { readFile, writeFile } from 'node:fs/promises';
// @ts-expect-error: tauri-version uses 'export =' which conflicts with ESM import
import tauri from 'tauri-version';

export default defineConfig({
  all: true,
  push: false,
  execute: async (ctx) => {
    await validateChangelog();

    await tauri({ lock: true })(ctx);
    await syncCargoLock();

    await updateChangelog(ctx.results);
  },
});

const syncCargoLock = async () => {
  await execa('cargo', ['update', '--workspace', '--offline'], {
    cwd: './src-tauri',
    stdio: 'inherit',
  });
};

const readChangelog = (path: string = './CHANGELOG.md') =>
  readFile(path, 'utf-8');

const writeChangelog = (data: string, path: string = './CHANGELOG.md') =>
  writeFile(path, data);

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
