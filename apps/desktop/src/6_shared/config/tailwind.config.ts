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
        new: {
          background: 'oklch(var(--new-background) / <alpha-value>)',
          foreground: 'oklch(var(--new-forgeround) / <alpha-value>)',
          muted: {
            DEFAULT: 'oklch(var(--new-muted) / <alpha-value>)',
            foreground: 'oklch(var(--new-muted-foreground) / <alpha-value>)',
          },
          popover: {
            DEFAULT: 'oklch(var(--new-popover) / <alpha-value>)',
            foreground: 'oklch(var(--new-popover-foreground) / <alpha-value>)',
          },
          border: {
            DEFAULT: 'oklch(var(--new-border) / 0.07)',
            dynamic: 'oklch(var(--new-border) / <alpha-value>)',
            foreground: 'oklch(var(--new-border-foreground) / <alpha-value>)',
          },
          input: {
            DEFAULT: 'oklch(var(--new-input) / <alpha-value>)',
            foreground: 'oklch(var(--new-input-foreground) / <alpha-value>)',
          },
          card: 'oklch(var(--new-card) / <alpha-value>)',
          primary: {
            DEFAULT: 'oklch(var(--new-primary) / <alpha-value>)',
            foreground: 'oklch(var(--new-primary-foreground) / <alpha-value>)',
          },
          secondary: {
            DEFAULT: 'oklch(var(--new-secondary) / <alpha-value>)',
            foreground:
              'oklch(var(--new-secondary-foreground) / <alpha-value>)',
          },
          accent: {
            DEFAULT: 'oklch(var(--new-accent) / <alpha-value>)',
            foreground: 'oklch(var(--new-accent-foreground) / <alpha-value>)',
          },
          destructive: {
            DEFAULT: 'oklch(var(--new-destructive) / <alpha-value>)',
            foreground:
              'oklch(var(--new-destructive-foreground) / <alpha-value>)',
          },
          info: {
            DEFAULT: 'oklch(var(--new-info) / <alpha-value>)',
            foreground: 'oklch(var(--new-info-foreground) / <alpha-value>)',
          },
          success: {
            DEFAULT: 'oklch(var(--new-success) / <alpha-value>)',
            foreground: 'oklch(var(--new-success-foreground) / <alpha-value>)',
          },
          warning: {
            DEFAULT: 'oklch(var(--new-warning) / <alpha-value>)',
            foreground: 'oklch(var(--new-warning-foreground) / <alpha-value>)',
          },
          error: {
            DEFAULT: 'oklch(var(--new-error) / <alpha-value>)',
            foreground: 'oklch(var(--new-error-foreground) / <alpha-value>)',
          },
          ring: {
            DEFAULT: 'oklch(var(--new-ring) / <alpha-value>)',
            foreground: 'oklch(var(--new-ring-foreground) / <alpha-value>)',
          },
        },
      },
      opacity: {
        card: '0.03',

        control: '0.0605',
        hover: '0.0903',
        active: '0.0302',

        'solid-hover': '0.9',
        'solid-active': '0.8',
        'solid-text-active': '0.7',

        secondary: '0.08',

        hard: '0.99',
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
