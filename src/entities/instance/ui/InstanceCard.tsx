import { splitProps } from 'solid-js';
import type { Component, ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type { Instance } from '@/entities/instance';

import type { InstanceActionButtonProps } from '@/features/instance-action-button';

import { InstanceImage } from './InstanceImage';
import { InstanceTitle } from './InstanceTitle';

export type InstanceCardProps = ComponentProps<'div'> & {
  instance: Instance;
  instanceActionButton: Component<InstanceActionButtonProps>;
};

export const InstanceCard: Component<InstanceCardProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'instanceActionButton',
    'class',
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
      <local.instanceActionButton
        class='absolute bottom-1/3 left-1/2 p-0 pr-0.5 opacity-0 transition-[bottom,opacity] disabled:opacity-0 group-hover:bottom-1/4 group-hover:opacity-100'
        instance={local.instance}
      />
    </div>
  );
};
