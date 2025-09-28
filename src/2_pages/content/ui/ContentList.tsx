import { For, splitProps, type Component, type ComponentProps } from 'solid-js';

import type { ContentItemExtended } from '@/entities/instances';

import { cn } from '@/shared/lib';

import { ContentListItem } from './ContentListItem';

export type ContentListProps = ComponentProps<'div'> & {
  items: ContentItemExtended[];
  instanceId: string;
  gameVersion: string;
  loader?: string;
  provider?: string;
  onInstalled?: (providerData: ContentItemExtended['providerData']) => void;
};

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
            item={item}
            instanceId={local.instanceId}
            gameVersion={local.gameVersion}
            loader={local.loader}
            provider={local.provider}
            onInstalled={local.onInstalled}
          />
        )}
      </For>
    </div>
  );
};
