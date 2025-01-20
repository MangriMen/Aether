import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { InstanceActionButton } from '../InstanceActionButton';
import { InstanceImage } from '../InstanceImage';
import { InstanceTitle } from '../InstanceTitle';

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
      <InstanceImage src={local.instance.iconPath} />
      <InstanceTitle
        name={local.instance.name}
        loader={local.instance.loader}
        gameVersion={local.instance.gameVersion}
      />
      <InstanceActionButton
        class='absolute bottom-1/3 left-1/2 p-0 pr-0.5 opacity-0 transition-[bottom,opacity] disabled:opacity-0 group-hover:bottom-1/4 group-hover:opacity-100'
        isLoading={local.isLoading}
        isRunning={local.isRunning}
        installStage={local.instance.installStage}
        onLaunchClick={local.onLaunchClick}
        onStopClick={local.onStopClick}
      />
    </div>
  );
};
