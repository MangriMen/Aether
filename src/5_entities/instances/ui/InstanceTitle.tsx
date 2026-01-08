import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

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

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <div class='w-full truncate'>{local.name}</div>
      <div class='inline-flex justify-between text-sm capitalize text-muted-foreground'>
        <span>{local.loader}</span>
        <span>{local.gameVersion}</span>
      </div>
    </div>
  );
};
