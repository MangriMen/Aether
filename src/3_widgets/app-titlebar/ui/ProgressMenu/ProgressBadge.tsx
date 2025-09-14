import { cn } from '@/shared/lib';
import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';

export type ProgressBadgeProps = ComponentProps<'div'>;

export const ProgressBadge: Component<ProgressBadgeProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <div
      class={cn(
        'w-full h-2 relative rounded-md overflow-hidden border border-secondary-foreground/5',
        local.class,
      )}
      {...others}
    >
      <div class='size-full bg-secondary' />
      <div class='absolute left-[-22px] top-[-18px] aspect-square w-[calc(100%*1.3)] animate-spin rounded-[110%_30%_80%_20%/40%_90%_40%_60%] bg-green-500 [animation-duration:12s]' />
    </div>
  );
};
