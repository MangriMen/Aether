import type { RouteSectionProps } from '@solidjs/router';
import type { Component } from 'solid-js';
import { onMount } from 'solid-js';

import { Toaster } from '@/shared/ui';

import { ColorModeProvider, I18nProvider, ThemeProvider } from './providers';
import { initializeResources, usePreventRightClick } from './lib';
import { useMaximizeObserver } from '@/shared/lib';
import { RAW_THEME_LS_KEY, THEME_ATTRIBUTE, THEME_LS_KEY } from './config';
import { LOCALE_RESOURCES, LOCALES } from '@/shared/model';
import { AppLayout } from './layouts/AppLayout';

export const AppRoot: Component<RouteSectionProps> = (props) => {
  usePreventRightClick();
  useMaximizeObserver();

  onMount(initializeResources);

  return (
    <ColorModeProvider {...props}>
      <ThemeProvider
        rawThemeLsKey={RAW_THEME_LS_KEY}
        themeLsKey={THEME_LS_KEY}
        themeAttribute={THEME_ATTRIBUTE}
      >
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <AppLayout>{props.children}</AppLayout>
          <Toaster />
        </I18nProvider>
      </ThemeProvider>
    </ColorModeProvider>
  );
};
