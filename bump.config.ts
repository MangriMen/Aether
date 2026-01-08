import { defineConfig } from 'bumpp';
import { execa } from 'execa';
// @ts-expect-error: tauri-version uses 'export =' which conflicts with ESM import
import tauri from 'tauri-version';

export default defineConfig({
  all: true,
  execute: async (ctx) => {
    await tauri({ lock: true })(ctx);
    syncCargoLock();
  },
});

const syncCargoLock = () => {
  execa('cargo', ['update', '--workspace', '--offline'], {
    cwd: './src-tauri',
    stdio: 'inherit',
  });
};
