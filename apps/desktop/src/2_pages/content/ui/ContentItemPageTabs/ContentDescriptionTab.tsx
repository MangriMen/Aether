import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { MarkdownRenderer } from '@/shared/ui';

import type { ContentItemPageTabProps } from '../../model/contentItemPageTabs';

export type ContentDescriptionTabProps = ComponentProps<'div'> &
  ContentItemPageTabProps;

export const ContentDescriptionTab: Component<ContentDescriptionTabProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  return (
    <div class={cn('', local.class)} {...others}>
      <Show when={local.item?.longDescription}>
        {(description) => (
          <div class='max-w-full overflow-auto rounded-md border border-secondary-dark bg-new-card/card dark:border-secondary'>
            <MarkdownRenderer class='p-4' children={description()} />
          </div>
        )}
      </Show>
    </div>
  );
};
