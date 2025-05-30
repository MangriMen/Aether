import type { RouteSectionProps } from '@solidjs/router';
import { type Component } from 'solid-js';

import { Toaster } from '@/shared/ui';

import { ColorModeProvider, I18nProvider, ThemeProvider } from './providers';
import {
  THEME_STATE_LS_KEY,
  THEME_ATTRIBUTE,
  TRANSPARENCY_PROPERTY,
} from './config';
import { LOCALE_RESOURCES, LOCALES } from '@/shared/model';
import { AppLayout } from './layouts/AppLayout';
import { AppInitializeGuard } from './providers/AppInitializeGuard/AppInitializeGuard';
import { useSetup } from './lib/useSetup';
import { MainLayout } from './layouts/MainLayout';
import { RunningInstancesProvider } from '@/entities/instances';

import { QueryClientProvider } from '@tanstack/solid-query';
import { createQueryClient } from '@/shared/api';
import { AppGlobalsProvider } from './providers/AppGlobalsProvider';

const queryClient = createQueryClient();

export const AppRoot: Component<RouteSectionProps> = (props) => {
  useSetup();

  return (
    <ColorModeProvider {...props}>
      <ThemeProvider
        themeStateKey={THEME_STATE_LS_KEY}
        themeAttribute={THEME_ATTRIBUTE}
        transparencyProperty={TRANSPARENCY_PROPERTY}
      >
        <AppInitializeGuard>
          <QueryClientProvider client={queryClient}>
            <I18nProvider
              resources={LOCALE_RESOURCES}
              fallbackLocale={LOCALES.En}
            >
              <AppGlobalsProvider>
                <AppLayout>
                  <RunningInstancesProvider>
                    <MainLayout>{props.children}</MainLayout>
                  </RunningInstancesProvider>
                </AppLayout>
                <Toaster />
              </AppGlobalsProvider>
            </I18nProvider>
          </QueryClientProvider>
        </AppInitializeGuard>
      </ThemeProvider>
    </ColorModeProvider>
  );
};
