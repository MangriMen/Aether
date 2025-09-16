import type { Component, ComponentProps } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import MdiClockIcon from '@iconify/icons-mdi/clock';
import { createMemo, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { formatTimePlayedHumanized } from '../lib';

export type LastPlayedDateProps = ComponentProps<'span'> & {
  lastPlayed: string | undefined;
  timePlayed: number;
};

export const TimePlayed: Component<LastPlayedDateProps> = (props) => {
  const [local, others] = splitProps(props, [
    'lastPlayed',
    'timePlayed',
    'class',
  ]);

  const [{ locale, t }] = useTranslation();

  const lastPlayedDate = createMemo(() => {
    return local.lastPlayed
      ? new Date(Date.parse(local.lastPlayed))
      : undefined;
  });

  const lastPlayedText = createMemo(() =>
    formatTimePlayedHumanized(local.timePlayed),
  );

  const lastPlayedDateTitle = createMemo(() =>
    t('instance.lastPlayed', {
      date: lastPlayedDate()?.toLocaleString(locale()) ?? '',
    }),
  );

  return (
    <span
      class={cn('mt-auto inline-flex items-center gap-1', local.class)}
      {...others}
    >
      <Icon icon={MdiClockIcon} />
      <Show when={local.lastPlayed} fallback={t('instance.neverPlayed')}>
        <span
          class='mt-auto inline-flex items-center gap-1 capitalize'
          title={lastPlayedDateTitle()}
        >
          {lastPlayedText()}
        </span>
      </Show>
    </span>
  );
};
