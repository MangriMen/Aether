import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentItemPageTabProps } from '../../model/contentItemPageTabs';

import { cn } from '../../../../shared/lib';
import { MarkdownRenderer } from '../../../../shared/ui';

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
          <div class='max-w-full overflow-auto rounded-md border bg-card/card'>
            <MarkdownRenderer class='p-4' children={description()} />
          </div>
        )}
      </Show>
    </div>
  );
};
