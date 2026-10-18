import type { StorybookConfig } from 'storybook-solidjs-vite';

const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
  addons: [
    '@storybook/addon-docs',
    '@storybook/addon-a11y',
    '@storybook/addon-themes',
  ],
  framework: 'storybook-solidjs-vite',
  // The app's vite.config.ts is loaded automatically (solid, tailwind, icons,
  // svg plugins and tsconfigPaths); only drop the type checker.
  viteFinal: (viteConfig) => ({
    ...viteConfig,
    plugins: (viteConfig.plugins ?? []).flat().filter((plugin) => {
      const name = plugin && 'name' in plugin ? plugin.name : '';
      return !name.startsWith('vite-plugin-checker');
    }),
  }),
};

export default config;
