import { Show, type Component } from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { Image } from '@/shared/ui';

export type ContentItemInfoProps = {
  item: ContentItem;
};

export const ContentItemInfo: Component<ContentItemInfoProps> = (props) => {
  return (
    <div class='flex gap-2'>
      <Image
        class='aspect-square size-24'
        src={props.item.iconUrl || undefined}
      />
      <div class='flex flex-col text-muted-foreground'>
        <span class='text-lg font-bold text-foreground'>
          <a href={props.item.url} target='_blank' rel='noreferrer'>
            {props.item.name}
          </a>
          &#32;
          <Show when={props.item.author}>
            {(author) => (
              <span class='text-base font-semibold text-muted-foreground'>
                by {author()}
              </span>
            )}
          </Show>
        </span>
        <span>{props.item.description}</span>
      </div>
    </div>
  );
};
