import type { Component } from 'solid-js';

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
          <a href={props.item.url} target='_blank'>
            {props.item.name}
          </a>{' '}
          <span class='text-base font-semibold text-muted-foreground'>
            by {props.item.author}
          </span>
        </span>
        <span>{props.item.description}</span>
      </div>
    </div>
  );
};
