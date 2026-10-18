import type { Component, ComponentProps } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import { formatBytes } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, Progress } from '@/shared/ui-kit';

import type { ProgressEvent } from '../model/progressEvent';

import { formatProgressBytes } from '../model/progressBytes';

export type ProgressCardProps = ComponentProps<'div'> & {
  payload: ProgressEvent;
};

export const ProgressCard: Component<ProgressCardProps> = (props) => {
  const [local, others] = splitProps(props, ['payload', 'class']);

  const [{ t, locale }] = useTranslation();

  const clampedValue = createMemo(() => local.payload.fraction ?? 1);

  const ownTitle = createMemo(() => {
    switch (local.payload.event.type) {
      case 'java_download':
        return t('events.javaDownloading', {
          version: local.payload.event.version,
        });
      case 'minecraft_version_download':
        return t('events.minecraftDownloading', {
          version: local.payload.event.version,
        });
      case 'minecraft_assets_download':
        return t('events.minecraftAssets', {
          count: local.payload.event.file_count,
        });
      case 'file_download':
        return local.payload.event.file_name;
      case 'minecraft_download':
      case 'instance_update':
      case 'zip_extract':
        return local.payload.event.instance_name;
      case 'pack_download':
      case 'pack_file_download':
        return t('events.installing', { name: local.payload.event.pack_name });
      case 'plugin_download':
        return `[Plugin] ${local.payload.event.plugin_name}`;
      case 'launcher_update':
        return t('events.launcherUpdating', {
          version: local.payload.event.version,
        });
      // default: {
      //   const _exhaustiveCheck: never = local.payload.event.type;
      //   return _exhaustiveCheck;
      // }
    }
  });

  // A mirror shows a shared download its install waits on.
  const title = createMemo(() =>
    local.payload.mirrorOf === null
      ? ownTitle()
      : t('events.waitingFor', { name: ownTitle() ?? '' }),
  );

  const message = createMemo(() => {
    const { status, stage, event } = local.payload;

    if (status.type === 'failed') {
      return status.message;
    }

    if (status.type === 'cancelled') {
      return t('events.cancelled');
    }

    if (event.type === 'launcher_update') {
      return t(`update.events.phase.${event.phase}`);
    }

    return stage ? t(`events.stage.${stage}`) : local.payload.message;
  });

  const isError = createMemo(
    () =>
      local.payload.status.type === 'failed' ||
      (local.payload.event.type === 'launcher_update' &&
        local.payload.event.phase === 'error'),
  );

  const percentStr = createMemo(() => `${Math.round(clampedValue() * 100)}%`);

  const bytesStr = createMemo(() =>
    formatProgressBytes(local.payload, {
      formatSize: (bytes) => formatBytes(bytes, locale()),
      formatSpeed: (speed) =>
        t('events.perSecond', { size: formatBytes(speed, locale()) }) ?? '',
    }),
  );

  return (
    <div class='gap-2 flex flex-col' {...others}>
      <div class='gap-4 flex items-end justify-between'>
        <CombinedTooltip
          label={title() ?? ''}
          as='span'
          class='font-bold truncate'
        >
          {title()}
        </CombinedTooltip>
        <span class='text-muted-foreground gap-2 flex shrink-0 tabular-nums'>
          <Show when={bytesStr()}>
            <span class='text-sm'>{bytesStr()}</span>
          </Show>
          {percentStr()}
        </span>
      </div>

      <Progress
        variant={isError() ? 'error' : undefined}
        value={clampedValue()}
        minValue={0}
        maxValue={1}
      />

      <span class='text-sm text-muted-foreground truncate'>{message()}</span>
    </div>
  );
};
