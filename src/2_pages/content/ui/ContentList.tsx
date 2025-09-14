import { type Component, type ComponentProps, For, splitProps } from 'solid-js';

import type { ContentItemExtended } from '@/entities/instances';

import { cn } from '@/shared/lib';

import { ContentListItem } from './ContentListItem';

export type ContentListProps = {
  gameVersion: string;
  instanceId: string;
  items: ContentItemExtended[];
  loader?: string;
  onInstalled?: (providerData: ContentItemExtended['providerData']) => void;
  provider?: string;
} & ComponentProps<'div'>;

export const ContentList: Component<ContentListProps> = (props) => {
  const [local, others] = splitProps(props, [
    'items',
    'instanceId',
    'gameVersion',
    'loader',
    'provider',
    'onInstalled',
    'class',
  ]);

  return (
    <div
      class={cn('flex flex-col gap-2 overflow-y-auto', local.class)}
      {...others}
    >
      <For each={local.items}>
        {(item) => (
          <ContentListItem
            gameVersion={local.gameVersion}
            instanceId={local.instanceId}
            item={item}
            loader={local.loader}
            onInstalled={local.onInstalled}
            provider={local.provider}
          />
        )}
      </For>
    </div>
  );
};
