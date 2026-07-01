import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type ProgressBadgeProps = ComponentProps<'div'>;

export const ProgressBadge: Component<ProgressBadgeProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <div
      class={cn(
        'relative h-2 w-full overflow-hidden rounded-md border border-secondary-foreground/5',
        local.class,
      )}
      {...others}
    >
      <div class='size-full bg-secondary/secondary' />
      <div class='absolute top-[-18px] left-[-22px] aspect-square w-[130%] animate-spin rounded-[110%_30%_80%_20%/40%_90%_40%_60%] bg-green-500 [animation-duration:12s]' />
    </div>
  );
};
