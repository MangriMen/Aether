import type { Component, ComponentProps } from 'solid-js';

import IconMdiClock from '~icons/mdi/clock';
import { createMemo, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

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
      <IconMdiClock />
      <Show when={local.lastPlayed} fallback={t('instance.neverPlayed')}>
        <CombinedTooltip
          as='span'
          label={lastPlayedDateTitle()}
          class='mt-auto inline-flex items-center gap-1 capitalize'
        >
          {lastPlayedText()}
        </CombinedTooltip>
      </Show>
    </span>
  );
};
