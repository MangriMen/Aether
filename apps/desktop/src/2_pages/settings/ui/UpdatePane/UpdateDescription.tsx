import type { Update } from '@tauri-apps/plugin-updater';
import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps, Show } from 'solid-js';

import { cn, dayjs, formatTime } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

export interface UpdateDescriptionProps extends ComponentProps<'div'> {
  isUpdateAvailable: boolean;
  update: Update | null | undefined;
}

export const UpdateDescription: Component<UpdateDescriptionProps> = (props) => {
  const [local, others] = splitProps(props, [
    'isUpdateAvailable',
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
      fallback={t('settings.checkForUpdatesDescriptionNoUpdates')}
    >
      <div class={cn('flex flex-col gap-0.5', local.class)} {...others}>
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
