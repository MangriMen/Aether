import { Icon } from '@iconify-icon/solid';
import { Component, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { IconButton, PlayIcon } from '@/shared/ui';

// eslint-disable-next-line boundaries/element-types
import { InstanceInstallStage } from '@/entities/minecraft';

import { InstanceCardProps } from './types';

export const InstanceCard: Component<InstanceCardProps> = (props) => {
  const [local, others] = splitProps(props as InstanceCardProps, [
    'class',
    'onLaunchClick',
    'instance',
    'isLoading',
  ]);

  return (
    <div
      class={cn(
        local.class,
        'group flex flex-col cursor-pointer gap-2 border rounded-md p-2 size-max overflow-hidden relative active:animate-bump-out',
      )}
      {...others}
    >
      <div class='size-24 rounded-lg border bg-secondary'>
        <Show
          when={local.instance.iconPath}
          fallback={<Icon class='text-[96px]' icon='mdi-cube-outline' />}
        >
          {(path) => <img src={path()} />}
        </Show>
      </div>
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
        class='absolute bottom-1/3 left-1/2 opacity-0 transition-[bottom,opacity] group-hover:bottom-1/4 group-hover:opacity-100'
        onClick={() => local.onLaunchClick?.()}
      >
        <Show
          when={
            local.instance.installStage === InstanceInstallStage.Installed &&
            !local.isLoading
          }
          fallback={<Icon class='animate-spin text-2xl' icon='mdi-loading' />}
        >
          <PlayIcon />
        </Show>
      </IconButton>
    </div>
  );
};
