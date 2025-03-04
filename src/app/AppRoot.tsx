import type { RouteSectionProps } from '@solidjs/router';
import { createSignal, onMount, Show, type Component } from 'solid-js';

import { Toaster } from '@/shared/ui';

import { ColorModeProvider, I18nProvider, ThemeProvider } from './providers';
import {
  initializeApp,
  initializeResources,
  usePreventRightClick,
} from './lib';
import { useMaximizeObserver } from '@/shared/lib';
import { RAW_THEME_LS_KEY, THEME_ATTRIBUTE, THEME_LS_KEY } from './config';
import { LOCALE_RESOURCES, LOCALES } from '@/shared/model';
import { AppLayout } from './layouts/AppLayout';
import { useInstanceEventsListener } from '@/entities/instances';
import { useWarningEventsListener } from '@/entities/events';
import { loadEnabledPlugins } from '@/entities/minecrafts';
import { refetchPlugins } from '@/entities/plugins';

export const AppRoot: Component<RouteSectionProps> = (props) => {
  const [isAppInitialized, setIsAppInitialized] = createSignal(false);

  usePreventRightClick();
  useMaximizeObserver();

  useInstanceEventsListener();
  useWarningEventsListener();

  onMount(() => {
    const init = async () => {
      await initializeApp();
      await initializeResources();
      setIsAppInitialized(true);

      await loadEnabledPlugins();
      await refetchPlugins();
    };
    init();
  });

  return (
    <Show when={isAppInitialized()}>
      <ColorModeProvider {...props}>
        <ThemeProvider
          rawThemeLsKey={RAW_THEME_LS_KEY}
          themeLsKey={THEME_LS_KEY}
          themeAttribute={THEME_ATTRIBUTE}
        >
          <I18nProvider
            resources={LOCALE_RESOURCES}
            fallbackLocale={LOCALES.En}
          >
            <AppLayout>{props.children}</AppLayout>
            <Toaster />
          </I18nProvider>
        </ThemeProvider>
      </ColorModeProvider>
    </Show>
  );
};
