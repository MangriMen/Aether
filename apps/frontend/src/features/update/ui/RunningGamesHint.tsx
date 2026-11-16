import type { Component } from 'solid-js';

import { Show } from 'solid-js';

import { useTranslation } from '@/shared/i18n';

export type RunningGamesHintProps = { games: string[] };

/** Why the update cannot be installed now, and what to close. */
export const RunningGamesHint: Component<RunningGamesHintProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Show when={props.games.length > 0}>
      <p class='text-sm text-muted-foreground' role='status'>
        {t('update.closeGameToInstall', { games: props.games.join(', ') })}
      </p>
    </Show>
  );
};
