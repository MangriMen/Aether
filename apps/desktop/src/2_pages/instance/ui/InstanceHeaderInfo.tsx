import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceGameVersion, TimePlayed } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { CombinedTooltip } from '@/shared/ui';

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
      <CombinedTooltip
        label={local.instance.name}
        as='span'
        class='line-clamp-2 text-2xl font-bold text-foreground [word-break:break-word]'
      >
        {local.instance.name}
      </CombinedTooltip>
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
