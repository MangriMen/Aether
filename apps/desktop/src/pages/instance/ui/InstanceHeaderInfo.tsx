import type { Component, ComponentProps } from 'solid-js';

import { mergeProps, Show, splitProps } from 'solid-js';

import type { Instance } from '../../../entities/instances';

import { InstanceGameVersion, TimePlayed } from '../../../entities/instances';
import { cn } from '../../../shared/lib';
import { CombinedTooltip, DelayedShow, Skeleton } from '../../../shared/ui';

export type InstanceHeaderInfoProps = ComponentProps<'div'> & {
  instance: Instance | undefined;
  showTimePlayed?: boolean;
};

export const InstanceHeaderInfo: Component<InstanceHeaderInfoProps> = (
  props,
) => {
  const merged = mergeProps({ showTimePlayed: true }, props);

  const [local, others] = splitProps(merged, [
    'instance',
    'showTimePlayed',
    'class',
  ]);

  return (
    <div
      class={cn(
        'flex flex-col text-muted-foreground justify-evenly',
        local.class,
      )}
      {...others}
    >
      <DelayedShow
        when={local.instance?.name}
        fallback={<Skeleton class='mb-1' width={96} height={20} radius={4} />}
      >
        {(name) => (
          <CombinedTooltip
            label={name()}
            as='h2'
            class='line-clamp-2 text-2xl font-bold text-foreground [word-break:break-word]'
          >
            {name()}
          </CombinedTooltip>
        )}
      </DelayedShow>

      <div class='flex gap-4'>
        <DelayedShow
          when={local.instance}
          fallback={<Skeleton width={128} height={16} radius={4} />}
        >
          {(instance) => (
            <InstanceGameVersion
              loader={instance().loader}
              gameVersion={instance().gameVersion}
            />
          )}
        </DelayedShow>

        <Show when={local.showTimePlayed}>
          <DelayedShow
            when={local.instance}
            fallback={<Skeleton width={128} height={16} radius={4} />}
          >
            {(instance) => (
              <TimePlayed
                lastPlayed={instance().lastPlayed ?? undefined}
                timePlayed={instance().timePlayed ?? undefined}
              />
            )}
          </DelayedShow>
        </Show>
      </div>
    </div>
  );
};
