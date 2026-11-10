import { createEffect, createMemo, createSignal, on } from 'solid-js';

import type { Option } from '@/shared/ui-kit';

import { useRunningInstancesContext } from '@/features/instance';
import { dayjs } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';

import { useGameLogSources } from '../api/gameLogApi';
import { LIVE_SOURCE as LIVE } from '../model/constants';

/** The logs of an instance as the options of one select, and which of them is shown. */
export const createLogSources = (instanceId: () => string) => {
  const [{ t }] = useTranslation();

  const sources = useGameLogSources(instanceId);
  const [running] = useRunningInstancesContext();
  const isRunning = () => running.instances[instanceId()]?.isRunning ?? false;

  // A run starting or ending adds or finishes a console log.
  createEffect(
    on(
      isRunning,
      () => {
        void sources.refetch();
      },
      { defer: true },
    ),
  );

  const [selected, setSelected] = createSignal<string>(LIVE);

  const newestConsole = createMemo(() =>
    sources.data?.find((source) => source.kind === 'console'),
  );

  const options = createMemo((): Option<string>[] => {
    const live = newestConsole();

    return (sources.data ?? []).map((source) => {
      if (source === live) {
        return {
          value: LIVE,
          name: (isRunning() ? t('logs.live') : t('logs.lastRun')) ?? '',
        };
      }

      const when = source.modified
        ? ` (${dayjs(source.modified).format('L LT')})`
        : '';

      return {
        value: source.id,
        name:
          source.kind === 'console'
            ? `${t('logs.earlierRun') ?? ''}${when}`
            : `${source.name}${when}`,
      };
    });
  });

  const currentOption = createMemo(() => {
    const all = options();

    return all.find((option) => option.value === selected()) ?? all[0];
  });

  const sourceId = createMemo(() => {
    const value = currentOption()?.value;

    return value === LIVE ? newestConsole()?.id : value;
  });

  const currentSource = createMemo(() =>
    sources.data?.find((source) => source.id === sourceId()),
  );

  return {
    sources,
    options,
    currentOption,
    sourceId,
    currentSource,
    select: setSelected,
  };
};
