import type { UserConfig } from 'vite';

import path from 'path';
import Icons from 'unplugin-icons/vite';
import { defineConfig } from 'vite';
import vitePluginChecker from 'vite-plugin-checker';
import solid from 'vite-plugin-solid';
import solidSvg from 'vite-plugin-solid-svg';

const host = process.env.TAURI_DEV_HOST;

const vitePluginCheckerConfig = vitePluginChecker({
  typescript: true,
  overlay: {
    initialIsOpen: false,
  },
});

const solidPluginConfig = solid({
  hot: process.env.NODE_ENV !== 'test',
});

const iconsConfig = Icons({
  compiler: 'solid',
  autoInstall: false,
});

const plugins = [
  vitePluginCheckerConfig,
  solidPluginConfig,
  iconsConfig,
  solidSvg(),
];

const srcAliases = {
  '@/app': path.resolve(__dirname, './src/app'),
  '@/pages': path.resolve(__dirname, './src/pages'),
  '@/widgets': path.resolve(__dirname, './src/widgets'),
  '@/features': path.resolve(__dirname, './src/features'),
  '@/entities': path.resolve(__dirname, './src/entities'),
  '@/shared': path.resolve(__dirname, './src/shared'),
};

const testConfig = {
  environment: 'jsdom',
  globals: true,
  transformMode: {
    web: [/\.[jt]sx?$/],
  },
  // isolates context for solid-js
  deps: {
    registerNodeLoader: false,
  },
  alias: srcAliases,
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
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
};

const buildConfig: UserConfig['build'] = {
  chunkSizeWarningLimit: 1500,
};

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  resolve: { tsconfigPaths: true },

  plugins: plugins,

  test: testConfig,
  build: buildConfig,

  ...developmentConfig,
}));
