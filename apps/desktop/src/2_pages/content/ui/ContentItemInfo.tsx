import type { ComponentProps } from 'solid-js';

import { Show, splitProps, type Component } from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { Image } from '@/shared/ui';

export type ContentItemInfoProps = {
  item: ContentItem;
};

export const ContentItemInfo: Component<
  ComponentProps<'div'> & ContentItemInfoProps
> = (props) => {
  const [local, others] = splitProps(props, ['item', 'class']);

  return (
    <div class={cn('flex gap-3', local.class)} {...others}>
      <Image
        class='aspect-square size-24 min-h-24 min-w-24 overflow-hidden rounded-md'
        src={local.item.iconUrl || undefined}
      />
      <div class='flex flex-col'>
        <span class='text-lg font-bold text-foreground'>
          <span class='group-hover:underline'>{local.item.name}</span>
          &#32;
          <Show when={local.item.author}>
            {(author) => (
              <span class='text-base font-semibold text-muted-foreground'>
                by {author()}
              </span>
            )}
          </Show>
        </span>
        <span class='text-muted-foreground'>{local.item.description}</span>
      </div>
    </div>
  );
};
