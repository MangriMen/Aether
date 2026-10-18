import type { Preview } from 'storybook-solidjs-vite';

import '../src/app/app.css';
import { THEMES } from '../src/features/appearance/model/theme';
import {
  KB_THEME_ATTRIBUTE,
  THEME_ATTRIBUTE,
} from '../src/features/appearance/model/themeConstants';
import { setThemeToDocument } from '../src/features/appearance/model/themeDom';

const preview: Preview = {
  globalTypes: {
    theme: {
      description: 'Aether theme',
      toolbar: {
        title: 'Theme',
        icon: 'paintbrush',
        items: THEMES.map((theme) => theme.value),
        dynamicTitle: true,
      },
    },
  },
  initialGlobals: { theme: 'aether-dark' },
  parameters: {
    layout: 'fullscreen',
    controls: { expanded: true },
  },
  decorators: [
    (Story, context) => {
      const theme = THEMES.find(
        (item) => item.value === context.globals['theme'],
      ) ?? { value: 'aether-dark', mode: 'dark' as const };
      setThemeToDocument(THEME_ATTRIBUTE, theme.value);
      const root = document.documentElement;
      root.setAttribute(KB_THEME_ATTRIBUTE, theme.mode);
      root.style.colorScheme = theme.mode;
      return (
        <div class='bg-background p-8 text-foreground'>
          <div
            class='
              min-h-96 gap-6 rounded-xl bg-card/card p-8 shadow-sm flex
              flex-wrap items-start border
            '
          >
            <Story />
          </div>
        </div>
      );
    },
  ],
};

export default preview;
