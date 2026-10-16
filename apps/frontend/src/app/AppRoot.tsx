import type { RouteSectionProps } from '@solidjs/router';

import { QueryClientProvider } from '@tanstack/solid-query';
import { type Component } from 'solid-js';

import { AppGlobalsProvider } from '@/app/runtime/AppGlobalsProvider';
import { AppInitializeGuard } from '@/app/runtime/AppInitializeGuard';
import { useSetup } from '@/app/runtime/useSetup';
import { AppLayout } from '@/app/shell/layouts/AppLayout';
import { MainLayout } from '@/app/shell/layouts/MainLayout';
import { RootErrorBoundary } from '@/app/shell/layouts/RootErrorBoundary';
import { AppTitleBar } from '@/app/shell/titlebar/AppTitleBar';
import { RunningInstancesProvider } from '@/entities/instances';
import { LOCALE_RESOURCES, LOCALES } from '@/shared/model';
import { createQueryClient } from '@/shared/query';
import { GlobalDialogRenderer, Toaster } from '@/shared/ui';

import {
  THEME_STATE_LS_KEY,
  THEME_ATTRIBUTE,
  TRANSPARENCY_PROPERTY,
  DISABLE_ANIMATIONS_ATTRIBUTE,
} from './config';
import { ColorModeProvider, I18nProvider, ThemeProvider } from './providers';
import { DialogUrlBridge } from './providers/DialogUrlBridge';

const queryClient = createQueryClient();

export const AppRoot: Component<RouteSectionProps> = (props) => {
  useSetup();

  return (
    <RootErrorBoundary>
      <ColorModeProvider>
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <ThemeProvider
            themeStateKey={THEME_STATE_LS_KEY}
            themeAttribute={THEME_ATTRIBUTE}
            disableAnimationsAttribute={DISABLE_ANIMATIONS_ATTRIBUTE}
            transparencyProperty={TRANSPARENCY_PROPERTY}
          >
            <AppInitializeGuard>
              <QueryClientProvider client={queryClient}>
                <AppGlobalsProvider>
                  <AppLayout titleBar={AppTitleBar}>
                    <RunningInstancesProvider>
                      <MainLayout>{props.children}</MainLayout>
                      <GlobalDialogRenderer />
                      <DialogUrlBridge />
                    </RunningInstancesProvider>
                  </AppLayout>
                  <Toaster />
                </AppGlobalsProvider>
              </QueryClientProvider>
            </AppInitializeGuard>
          </ThemeProvider>
        </I18nProvider>
      </ColorModeProvider>
    </RootErrorBoundary>
  );
};
