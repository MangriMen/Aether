import type { Config } from 'tailwindcss/types/config';

import typographyPlugin from '@tailwindcss/typography';
import resolveConfig from 'tailwindcss/resolveConfig';

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
      keyframes: {
        'content-show': {
          from: { opacity: '0', transform: 'scaleY(0) translateY(-40px)' },
          to: { opacity: '1', transform: 'scaleY(1) translateY(0)' },
        },
        'content-hide': {
          from: { opacity: '1', transform: 'scaleY(1) translateY(0)' },
          to: { opacity: '0', transform: 'scaleY(0) translateY(-40px)' },
        },
        'slide-down': {
          from: { height: '0' },
          to: { height: 'var(--kb-collapsible-content-height)' },
        },
        'slide-up': {
          from: { height: 'var(--kb-collapsible-content-height)' },
          to: { height: '0' },
        },
        'bump-out': {
          '0%': {
            transform: 'scale(1)',
          },
          '100%': {
            transform: 'scale(0.95)',
          },
        },
        'bump-in': {
          '0%': {
            transform: 'scale(1)',
          },
          '100%': {
            transform: 'scale(1.05)',
          },
        },
      },
      animation: {
        'bump-in': 'bump-in 0.05s ease-in-out forwards',
        'bump-out': 'bump-out 0.05s ease-in-out forwards',
        'content-show': 'content-show 0.2s ease-out',
        'content-hide': 'content-hide 0.2s ease-out',
        'slide-down': 'slide-down 0.2s ease-out',
        'slide-up': 'slide-up 0.2s ease-out',
      },
    },
  },

  plugins: [typographyPlugin],
};

export const resolvedTailwindConfig = resolveConfig(tailwindConfig);
