import { defineConfig } from 'bumpp';
import * as tauri from 'tauri-version';

export default defineConfig({
  all: true,
  execute: async (ctx) => {
    await tauri()(ctx);
  },
});
