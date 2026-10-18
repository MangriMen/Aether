import type { Preview } from 'storybook-solidjs-vite';

import { QueryClientProvider } from '@tanstack/solid-query';

import '../src/app/app.css';
import { THEMES } from '../src/features/appearance/model/theme';
import './storybook.css';
import {
  KB_THEME_ATTRIBUTE,
  THEME_ATTRIBUTE,
} from '../src/features/appearance/model/themeConstants';
import { setThemeToDocument } from '../src/features/appearance/model/themeDom';
import { I18nProvider, LOCALE_RESOURCES, LOCALES } from '../src/shared/i18n';
import { createQueryClient } from '../src/shared/query';
import { ThemedDocsContainer } from './DocsContainer';

const queryClient = createQueryClient();

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
    docs: { container: ThemedDocsContainer },
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
            <I18nProvider
              resources={LOCALE_RESOURCES}
              fallbackLocale={LOCALES.En}
            >
              <QueryClientProvider client={queryClient}>
                <Story />
              </QueryClientProvider>
            </I18nProvider>
          </div>
        </div>
      );
    },
  ],
};

export default preview;
