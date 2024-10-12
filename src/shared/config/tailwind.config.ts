import resolveConfig from 'tailwindcss/resolveConfig';
import { Config } from 'tailwindcss/types/config';

//@ts-expect-error cannot find a declaration
import presets from './ui.presets.cjs';

export const tailwindConfig: Config = {
  darkMode: ['class', '[data-kb-theme="dark"]'],
  content: ['./src/**/*.{html,js,jsx,md,mdx,ts,tsx}'],
  presets: [presets],
  theme: {
    extend: {
      colors: {
        secondary: {
          dark: 'hsl(var(--secondary-dark))',
        },
      },
    },
  },
};

export const resolvedTailwindConfig = resolveConfig(tailwindConfig);
