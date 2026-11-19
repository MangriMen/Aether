import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps, Show } from 'solid-js';

import { dayjs, formatTime } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { cn } from '@/shared/utils';

import type { UpdateStatus } from '../model/update';

export interface UpdateDescriptionProps extends ComponentProps<'div'> {
  isUpdateAvailable: boolean;
  /** The first check is under way: there is nothing to say about updates yet. */
  isChecking?: boolean;
  /** The last check failed, so "no updates" would be a guess. */
  isFailed?: boolean;
  update: UpdateStatus | null | undefined;
}

export const UpdateDescription: Component<UpdateDescriptionProps> = (props) => {
  const [local, others] = splitProps(props, [
    'isUpdateAvailable',
    'isChecking',
    'isFailed',
    'update',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const version = createMemo(() => local.update?.version);
  const date = createMemo(() =>
    local.update?.date ? formatTime(dayjs(local.update.date)) : undefined,
  );

  return (
    <Show
      when={local.isUpdateAvailable}
      fallback={
        <Show
          when={!local.isChecking}
          fallback={t('settings.checkForUpdatesDescriptionChecking')}
        >
          {local.isFailed
            ? t('settings.checkForUpdatesDescriptionFailed')
            : t('settings.checkForUpdatesDescriptionNoUpdates')}
        </Show>
      }
    >
      <div class={cn('gap-0.5 flex flex-col', local.class)} {...others}>
        {t('settings.checkForUpdatesDescription')}
        <Show when={version()}>
          {(version) => (
            <span>
              {t('common.version')}: {version()}
            </span>
          )}
        </Show>
        <Show when={date()}>
          {(date) => (
            <span>
              {t('settings.releaseDate')}: {date()}
            </span>
          )}
        </Show>
      </div>
    </Show>
  );
};
