import MdiClockIcon from '@iconify/icons-mdi/clock';
import MdiEngineIcon from '@iconify/icons-mdi/engine';
import MdiGamepadSquare from '@iconify/icons-mdi/gamepad-square';
import { Icon } from '@iconify-icon/solid';
import {
  Component,
  ComponentProps,
  createMemo,
  Show,
  splitProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { Separator } from '@/shared/ui';

import { formatTimePlayedHumanized, Instance } from '@/entities/instance';

export type InstanceHeaderInfoProps = ComponentProps<'div'> & {
  instance: Instance;
};

const InstanceHeaderInfo: Component<InstanceHeaderInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const lastPlayedDate = createMemo(() => {
    return local.instance?.lastPlayed
      ? new Date(Date.parse(local.instance?.lastPlayed))
      : undefined;
  });

  return (
    <div
      class={cn('flex flex-col text-muted-foreground', local.class)}
      {...others}
    >
      <span class='text-2xl font-bold text-foreground'>
        {local.instance.name}
      </span>
      <span class='inline-flex items-center gap-2 capitalize'>
        <span class='inline-flex items-center gap-1' title='Game version'>
          <Icon icon={MdiGamepadSquare} />
          {local.instance.gameVersion}
        </span>
        <Separator orientation='vertical' />
        <span class='inline-flex items-center gap-1' title='Modloader'>
          <Icon icon={MdiEngineIcon} />
          {local.instance.loader} {local.instance.loaderVersion}
        </span>
      </span>
      <span class='mt-auto inline-flex items-center gap-1'>
        <Icon icon={MdiClockIcon} />
        <Show when={local.instance.timePlayed} fallback='Never played'>
          <span
            class='mt-auto inline-flex items-center gap-1 capitalize'
            title={`Last played: ${lastPlayedDate()?.toLocaleString()}`}
          >
            {formatTimePlayedHumanized(local.instance.timePlayed)}
          </span>
        </Show>
      </span>
    </div>
  );
};

export default InstanceHeaderInfo;
