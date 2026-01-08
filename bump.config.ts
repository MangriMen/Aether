import { defineConfig } from 'bumpp';
// @ts-expect-error: tauri-version uses 'export =' which conflicts with ESM import
import tauri from 'tauri-version';

export default defineConfig({
  all: true,
  execute: tauri(),
});
