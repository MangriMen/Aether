import {
  For,
  splitProps,
  type Component,
  type ComponentProps,
  Switch,
  Match,
} from 'solid-js';

import type { ContentItem } from '@/features/content';

import { useSettingsSearchParams, SettingsTab } from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { Button } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { ContentListItem } from './ContentListItem';
import { ContentListSkeleton } from './ContentListSkeleton';

export type ContentListProps = ComponentProps<'div'> & {
  items?: ContentItem[];
  isLoading?: boolean;
  isError?: boolean;
  isProvidersError?: boolean;
  hasNoProviders?: boolean;
};

export const ContentList: Component<ContentListProps> = (props) => {
  const [local, others] = splitProps(props, [
    'items',
    'isLoading',
    'isError',
    'isProvidersError',
    'hasNoProviders',
    'class',
  ]);

  const [{ t }] = useTranslation();
  const settingsParams = useSettingsSearchParams();

  return (
    <div
      class={cn('gap-3 flex flex-col overflow-y-auto', local.class)}
      {...others}
    >
      <Switch>
        <Match when={local.isLoading}>
          <ContentListSkeleton />
        </Match>

        <Match when={local.isProvidersError}>
          <span
            class='
              text-xl font-medium text-muted-foreground flex grow items-center
              justify-center
            '
          >
            {t('content.providersError')}
          </span>
        </Match>

        <Match when={local.hasNoProviders}>
          <div class='gap-3 flex grow flex-col items-center justify-center'>
            <span class='text-lg text-muted-foreground'>
              {t('content.noProviders')}
            </span>
            <Button
              variant='secondary'
              onClick={() => settingsParams.open(SettingsTab.Plugins)}
            >
              {t('content.openPluginSettings')}
            </Button>
          </div>
        </Match>

        <Match when={local.isError}>
          <span
            class='
              text-xl font-medium text-muted-foreground flex grow items-center
              justify-center
            '
          >
            {t('content.providerErrorOrNotFound')}
          </span>
        </Match>

        <Match when={!local.items || local.items.length === 0}>
          <span
            class='
              text-lg text-muted-foreground flex grow items-center
              justify-center italic
            '
          >
            {t('content.noResultsFound')}
          </span>
        </Match>

        <Match when={local.items}>
          {(items) => (
            <For each={items()}>
              {(item) => <ContentListItem item={item} />}
            </For>
          )}
        </Match>
      </Switch>
    </div>
  );
};
