import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceGameVersion, TimePlayed } from '@/entities/instances';
import { cn } from '@/shared/lib';

export type InstanceHeaderInfoProps = ComponentProps<'div'> & {
  instance: Instance;
};

export const InstanceHeaderInfo: Component<InstanceHeaderInfoProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  return (
    <div
      class={cn('flex flex-col text-muted-foreground', local.class)}
      {...others}
    >
      <span class='text-2xl font-bold text-foreground'>
        {local.instance.name}
      </span>
      <InstanceGameVersion
        loader={local.instance.loader}
        gameVersion={local.instance.gameVersion}
      />
      <TimePlayed
        lastPlayed={local.instance.lastPlayed}
        timePlayed={local.instance.timePlayed}
      />
    </div>
  );
};
