import { For, splitProps, type Component, type ComponentProps } from 'solid-js';

import type { ContentItem } from '@/entities/instances';

import { cn } from '@/shared/lib';

import { ContentListItem } from './ContentListItem';

export type ContentListProps = ComponentProps<'div'> & {
  items: ContentItem[];
  instanceId?: string;
};

export const ContentList: Component<ContentListProps> = (props) => {
  const [local, others] = splitProps(props, ['items', 'instanceId', 'class']);

  return (
    <div
      class={cn('flex flex-col gap-2 overflow-y-auto', local.class)}
      {...others}
    >
      <For each={local.items}>
        {(item) => (
          <ContentListItem item={item} instanceId={local.instanceId} />
        )}
      </For>
    </div>
  );
};
