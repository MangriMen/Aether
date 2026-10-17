import type { RouteSectionProps } from '@solidjs/router';

import { QueryClientProvider } from '@tanstack/solid-query';
import { type Component } from 'solid-js';

import {
  AppGlobalsProvider,
  AppInitializeGuard,
  UiKitLabelsFromI18n,
  useSetup,
} from '@/app/runtime';
import {
  AppLayout,
  AppTitleBar,
  DialogUrlBridge,
  MainLayout,
  RootErrorBoundary,
} from '@/app/shell';
import {
  ColorModeProvider,
  DISABLE_ANIMATIONS_ATTRIBUTE,
  THEME_ATTRIBUTE,
  THEME_STATE_LS_KEY,
  ThemeProvider,
  TRANSPARENCY_PROPERTY,
} from '@/features/appearance';
import { RunningInstancesProvider } from '@/features/instance';
import { GlobalDialogRenderer } from '@/shared/dialogs';
import { LOCALE_RESOURCES, LOCALES, I18nProvider } from '@/shared/i18n';
import { createQueryClient } from '@/shared/query';
import { Toaster } from '@/shared/ui-kit';

const queryClient = createQueryClient();

export const AppRoot: Component<RouteSectionProps> = (props) => {
  useSetup();

  return (
    <RootErrorBoundary>
      <ColorModeProvider>
        <I18nProvider resources={LOCALE_RESOURCES} fallbackLocale={LOCALES.En}>
          <UiKitLabelsFromI18n>
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
          </UiKitLabelsFromI18n>
        </I18nProvider>
      </ColorModeProvider>
    </RootErrorBoundary>
  );
};
