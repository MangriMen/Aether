import { defineConfig } from 'vite';
import vitePluginChecker from 'vite-plugin-checker';
import solid from 'vite-plugin-solid';
import solidSvg from 'vite-plugin-solid-svg';
import tsconfigPaths from 'vite-tsconfig-paths';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,

  plugins: [
    vitePluginChecker({
      overlay: {
        initialIsOpen: false,
      },
      typescript: true,
    }),
    solid(),
    solidSvg(),
    tsconfigPaths(),
  ],
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}));
