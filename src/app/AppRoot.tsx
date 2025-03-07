import type { RouteSectionProps } from '@solidjs/router';
import { type Component } from 'solid-js';

import { Toaster } from '@/shared/ui';

import { ColorModeProvider, I18nProvider, ThemeProvider } from './providers';
import { RAW_THEME_LS_KEY, THEME_ATTRIBUTE, THEME_LS_KEY } from './config';
import { LOCALE_RESOURCES, LOCALES } from '@/shared/model';
import { AppLayout } from './layouts/AppLayout';
import { AppProvider } from './providers/AppProvider/AppProvider';
import { useSetup } from './lib/useSetup';
import { MainLayout } from './layouts/MainLayout';
import { RunningInstancesProvider } from '@/entities/instances';

export const AppRoot: Component<RouteSectionProps> = (props) => {
  useSetup();

  return (
    <ColorModeProvider {...props}>
      <ThemeProvider
        rawThemeLsKey={RAW_THEME_LS_KEY}
        themeLsKey={THEME_LS_KEY}
        themeAttribute={THEME_ATTRIBUTE}
      >
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <AppLayout>
            <AppProvider>
              <RunningInstancesProvider>
                <MainLayout>{props.children}</MainLayout>
              </RunningInstancesProvider>
            </AppProvider>
          </AppLayout>
          <Toaster />
        </I18nProvider>
      </ThemeProvider>
    </ColorModeProvider>
  );
};
