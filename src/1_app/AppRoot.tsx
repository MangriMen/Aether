import type { RouteSectionProps } from '@solidjs/router';

import { QueryClientProvider } from '@tanstack/solid-query';
import { type Component } from 'solid-js';

import { RunningInstancesProvider } from '@/entities/instances';
import { createQueryClient } from '@/shared/api';
import { LOCALE_RESOURCES, LOCALES } from '@/shared/model';
import { GlobalDialogRenderer, Toaster } from '@/shared/ui';

import {
  DISABLE_ANIMATIONS_ATTRIBUTE,
  THEME_ATTRIBUTE,
  THEME_STATE_LS_KEY,
  TRANSPARENCY_PROPERTY,
} from './config';
import { AppLayout } from './layouts/AppLayout';
import { MainLayout } from './layouts/MainLayout';
import { useSetup } from './lib/useSetup';
import { ColorModeProvider, I18nProvider, ThemeProvider } from './providers';
import { AppGlobalsProvider } from './providers/AppGlobalsProvider';
import { AppInitializeGuard } from './providers/AppInitializeGuard/AppInitializeGuard';

const queryClient = createQueryClient();

export const AppRoot: Component<RouteSectionProps> = (props) => {
  useSetup();

  return (
    <ColorModeProvider {...props}>
      <ThemeProvider
        disableAnimationsAttribute={DISABLE_ANIMATIONS_ATTRIBUTE}
        themeAttribute={THEME_ATTRIBUTE}
        themeStateKey={THEME_STATE_LS_KEY}
        transparencyProperty={TRANSPARENCY_PROPERTY}
      >
        <I18nProvider fallbackLocale={LOCALES.En} resources={LOCALE_RESOURCES}>
          <AppInitializeGuard>
            <QueryClientProvider client={queryClient}>
              <AppGlobalsProvider>
                <AppLayout>
                  <RunningInstancesProvider>
                    <MainLayout>{props.children}</MainLayout>
                    <GlobalDialogRenderer />
                  </RunningInstancesProvider>
                </AppLayout>
                <Toaster />
              </AppGlobalsProvider>
            </QueryClientProvider>
          </AppInitializeGuard>
        </I18nProvider>
      </ThemeProvider>
    </ColorModeProvider>
  );
};
