import type { ComponentProps } from 'solid-js';

import { For, splitProps } from 'solid-js';

import type { BadgeProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { Badge } from '@/shared/ui';

export interface ContentVersionBadgeListProps {
  items: string[];
  badgeProps?: BadgeProps;
}

export const ContentVersionBadgeList = (
  props: ComponentProps<'div'> & ContentVersionBadgeListProps,
) => {
  const [local, others] = splitProps(props, ['badgeProps', 'class']);

  return (
    <div class={cn('flex flex-wrap gap-1', local.class)} {...others}>
      <For each={props.items}>
        {(item) => (
          <Badge variant='secondary' {...local.badgeProps}>
            {item}
          </Badge>
        )}
      </For>
    </div>
  );
};
