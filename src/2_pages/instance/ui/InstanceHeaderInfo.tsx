import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceGameVersion, TimePlayed } from '@/entities/instances';
import { cn } from '@/shared/lib';

export type InstanceHeaderInfoProps = {
  instance: Instance;
} & ComponentProps<'div'>;

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
        gameVersion={local.instance.gameVersion}
        loader={local.instance.loader}
      />
      <TimePlayed
        lastPlayed={local.instance.lastPlayed}
        timePlayed={local.instance.timePlayed}
      />
    </div>
  );
};
