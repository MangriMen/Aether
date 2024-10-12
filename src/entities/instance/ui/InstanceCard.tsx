import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { IconButton, PlayIcon } from '@/shared/ui';

import { InstanceCardProps } from './types';

export const InstanceCard: Component<InstanceCardProps> = (props) => {
  const [local, others] = splitProps(props as InstanceCardProps, [
    'class',
    'onLaunchClick',
    'instance',
  ]);

  return (
    <div
      class={cn(
        local.class,
        'group flex flex-col gap-2 border rounded-md p-2 size-max overflow-hidden relative',
      )}
      {...others}
    >
      <div class='size-24 rounded-md border' />
      <div>
        <div>{local.instance.name}</div>
        <div class='text-sm capitalize text-muted-foreground'>
          <span>{local.instance.loader}</span>
          &nbsp;
          <span>{local.instance.gameVersion}</span>
        </div>
      </div>
      <IconButton
        variant='success'
        class='absolute bottom-1/3 left-1/2 rounded-md opacity-0 transition-[bottom,opacity] group-hover:bottom-1/4 group-hover:opacity-100'
        onClick={() => local.onLaunchClick?.()}
      >
        <PlayIcon />
      </IconButton>
    </div>
  );
};
