import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn, useIsOverflow } from '@/shared/lib';
import { CombinedTooltip } from '@/shared/ui';

export type InstanceTitleProps = ComponentProps<'div'> & {
  name?: string;
  loader?: string;
  gameVersion?: string;
};

export const InstanceTitle: Component<InstanceTitleProps> = (props) => {
  const [local, others] = splitProps(props, [
    'name',
    'loader',
    'gameVersion',
    'class',
  ]);

  const [isOverflowed, ref, updateOverflow] = useIsOverflow();

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <CombinedTooltip
        label={local.name}
        as='div'
        ref={ref}
        onPointerEnter={updateOverflow}
        class='w-full truncate'
        disableTooltip={!isOverflowed()}
      >
        {local.name}
      </CombinedTooltip>
      <div class='inline-flex justify-between text-sm capitalize text-muted-foreground'>
        <span>{local.loader}</span>
        <span>{local.gameVersion}</span>
      </div>
    </div>
  );
};
