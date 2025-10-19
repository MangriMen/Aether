import type { UserConfig } from 'vite';

import Icons from 'unplugin-icons/vite';
import { defineConfig } from 'vite';
import vitePluginChecker from 'vite-plugin-checker';
import solid from 'vite-plugin-solid';
import solidSvg from 'vite-plugin-solid-svg';
import tsconfigPaths from 'vite-tsconfig-paths';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vitePluginCheckerConfig,
    solid(),
    iconsConfig,
    solidSvg(),
    tsconfigPaths(),
  ],
  build: buildConfig,
  ...developmentConfig,
}));

const vitePluginCheckerConfig = vitePluginChecker({
  typescript: true,
  overlay: {
    initialIsOpen: false,
  },
});

const iconsConfig = Icons({
  compiler: 'solid',
  autoInstall: false,
});

const buildConfig: UserConfig['build'] = {
  chunkSizeWarningLimit: 1500,
};

const developmentConfig: UserConfig = {
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
};
