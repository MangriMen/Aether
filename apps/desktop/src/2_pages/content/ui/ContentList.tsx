import {
  For,
  splitProps,
  type Component,
  type ComponentProps,
  Switch,
  Match,
} from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { ContentListItem } from './ContentListItem';
import { ContentListSkeleton } from './ContentListSkeleton';

export type ContentListProps = ComponentProps<'div'> & {
  items?: ContentItem[];
  isLoading?: boolean;
  isError?: boolean;
};

export const ContentList: Component<ContentListProps> = (props) => {
  const [local, others] = splitProps(props, [
    'items',
    'isLoading',
    'isError',
    'class',
  ]);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn('flex flex-col gap-2 overflow-y-auto', local.class)}
      {...others}
    >
      <Switch>
        <Match when={local.isLoading}>
          <ContentListSkeleton />
        </Match>

        <Match when={local.isError}>
          <span class='flex grow items-center justify-center text-xl font-medium text-new-muted-foreground'>
            {t('content.providerErrorOrNotFound')}
          </span>
        </Match>

        <Match when={local.items?.length === 0}>
          <span class='flex grow items-center justify-center text-lg italic text-new-muted-foreground'>
            {t('content.noResultsFound')}
          </span>
        </Match>

        <Match when={local?.items}>
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
