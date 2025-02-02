import MdiClockIcon from '@iconify/icons-mdi/clock';
import MdiEngineIcon from '@iconify/icons-mdi/engine';
import MdiGamepadSquare from '@iconify/icons-mdi/gamepad-square';
import { Icon } from '@iconify-icon/solid';
import type { Component, ComponentProps } from 'solid-js';
import { createMemo, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTooltip, Separator } from '@/shared/ui';

import type { Instance } from '@/entities/instances';
import { formatTimePlayedHumanized } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

export type InstanceHeaderInfoProps = ComponentProps<'div'> & {
  instance: Instance;
};

const InstanceHeaderInfo: Component<InstanceHeaderInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ locale, t }] = useTranslate();

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
        <CombinedTooltip
          label={t('common.gameVersion')}
          as='span'
          class='inline-flex items-center gap-1'
        >
          <Icon icon={MdiGamepadSquare} />
          {local.instance.gameVersion}
        </CombinedTooltip>
        <Separator orientation='vertical' />
        <CombinedTooltip
          label={t('common.loader')}
          as='span'
          class='inline-flex items-center gap-1'
        >
          <Icon icon={MdiEngineIcon} />
          {local.instance.loader} {local.instance.loaderVersion}
        </CombinedTooltip>
      </span>
      <span class='mt-auto inline-flex items-center gap-1'>
        <Icon icon={MdiClockIcon} />
        <Show when={lastPlayedDate()} fallback={t('instance.neverPlayed')}>
          {(lastPlayedDate) => (
            <span
              class='mt-auto inline-flex items-center gap-1 capitalize'
              title={t('instance.lastPlayed', {
                date: lastPlayedDate().toLocaleString(locale()),
              })}
            >
              {formatTimePlayedHumanized(local.instance.timePlayed)}
            </span>
          )}
        </Show>
      </span>
    </div>
  );
};

export default InstanceHeaderInfo;
