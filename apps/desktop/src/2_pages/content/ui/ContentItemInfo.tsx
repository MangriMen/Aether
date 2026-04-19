import type { ComponentProps } from 'solid-js';

import { A } from '@solidjs/router';
import { Show, splitProps, type Component } from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { cn } from '@/shared/lib';
import { Image } from '@/shared/ui';

export type ContentItemInfoProps = {
  item: ContentItem;
  contentPageHref?: string;
};

export const ContentItemInfo: Component<
  ComponentProps<'div'> & ContentItemInfoProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'item',
    'contentPageHref',
    'class',
  ]);

  return (
    <div class={cn('flex gap-2', local.class)} {...others}>
      <Image
        class='aspect-square size-24'
        src={local.item.iconUrl || undefined}
      />
      <div class='flex flex-col'>
        <span class='text-lg font-bold text-new-foreground'>
          <Show
            when={local.contentPageHref}
            fallback={<span>{local.item.name}</span>}
          >
            {(href) => (
              <A class='group-hover:underline' href={href()}>
                {local.item.name}
              </A>
            )}
          </Show>
          &#32;
          <Show when={local.item.author}>
            {(author) => (
              <span class='text-base font-semibold text-new-muted-foreground'>
                by {author()}
              </span>
            )}
          </Show>
        </span>
        <span class='text-new-muted-foreground'>{local.item.description}</span>
      </div>
    </div>
  );
};
