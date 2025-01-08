import { Icon } from '@iconify-icon/solid';
import { Component, Match, Show, splitProps, Switch } from 'solid-js';

import { cn } from '@/shared/lib';
import { PlayIcon } from '@/shared/ui';

// eslint-disable-next-line boundaries/element-types
import { InstanceInstallStage } from '@/entities/minecraft';

import { InstanceButton } from '../InstanceButton';

import { InstanceCardProps } from './types';

export const InstanceCard: Component<InstanceCardProps> = (props) => {
  const [local, others] = splitProps(props, [
    'class',
    'onLaunchClick',
    'onStopClick',
    'instance',
    'isLoading',
    'isRunning',
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
      <Switch
        fallback={
          <InstanceButton
            variant={local.isRunning ? 'destructive' : 'success'}
            title={local.isRunning ? 'Stop' : 'Launch'}
          >
            <Icon class='animate-spin text-2xl' icon='mdi-loading' />
          </InstanceButton>
        }
      >
        <Match
          when={
            local.instance.installStage === InstanceInstallStage.Installed &&
            !local.isRunning &&
            !local.isLoading
          }
        >
          <InstanceButton
            variant='success'
            onClick={local.onLaunchClick}
            title='Launch'
          >
            <PlayIcon />
          </InstanceButton>
        </Match>
        <Match
          when={
            local.instance.installStage === InstanceInstallStage.Installed &&
            local.isRunning
          }
        >
          <InstanceButton
            variant='destructive'
            onClick={local.onStopClick}
            title='Stop'
          >
            <Icon icon='mdi-stop' />
          </InstanceButton>
        </Match>
      </Switch>
    </div>
  );
};
