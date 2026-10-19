import type { Component, ComponentProps } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import { formatBytes } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, Progress, useIsOverflow } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { ProgressEvent } from '../model/progressEvent';

import { formatProgressBytes } from '../model/progressBytes';

export type ProgressCardProps = ComponentProps<'div'> & {
  payload: ProgressEvent;
  /** `sm` is a tight single-row variant for nested bars. */
  size?: 'default' | 'sm';
};

export const ProgressCard: Component<ProgressCardProps> = (props) => {
  const [local, others] = splitProps(props, ['payload', 'size', 'class']);

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
        return t('events.gameInstalling');
      case 'instance_update':
      case 'zip_extract':
        return local.payload.event.instance_name;
      case 'pack_download':
        return t('events.installing', { name: local.payload.event.pack_name });
      case 'pack_file_download':
        return local.payload.event.pack_name;
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

    if (status.type === 'done') {
      return t('events.completed');
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

  const [isTitleOverflowed, titleRef, updateTitleOverflow] = useIsOverflow();

  const percentStr = createMemo(() => `${Math.round(clampedValue() * 100)}%`);

  const bytesStr = createMemo(() =>
    formatProgressBytes(local.payload, {
      formatSize: (bytes) => formatBytes(bytes, locale()),
      // A fixed fraction keeps the speed from jumping between "3 MB/s" and "3.1 MB/s".
      formatSpeed: (speed) =>
        t('events.perSecond', {
          size: formatBytes(speed, locale(), { fractionDigits: 1 }),
        }) ?? '',
    }),
  );

  const isSmall = () => local.size === 'sm';

  // Compact cards only spend a line on what needs attention.
  const showMessage = createMemo(
    () =>
      !isSmall() ||
      isError() ||
      local.payload.status.type === 'cancelled' ||
      local.payload.mirrorOf !== null,
  );

  return (
    <div
      class={cn('flex flex-col', isSmall() ? 'gap-1' : 'gap-2', local.class)}
      {...others}
    >
      <div class='gap-4 flex items-center justify-between'>
        <CombinedTooltip
          label={title() ?? ''}
          as='span'
          ref={titleRef}
          onPointerEnter={updateTitleOverflow}
          disableTooltip={!isTitleOverflowed()}
          class={cn('truncate', isSmall() ? 'text-sm' : 'font-bold')}
        >
          {title()}
        </CombinedTooltip>
        <span
          class={cn(
            'text-muted-foreground gap-2 flex shrink-0 items-center tabular-nums',
            { 'text-xs': isSmall() },
          )}
        >
          <Show when={isSmall() && bytesStr()}>
            <span>{bytesStr()}</span>
          </Show>
          {percentStr()}
        </span>
      </div>

      <Progress
        class={cn({ '[&>div]:h-1': isSmall() })}
        variant={isError() ? 'error' : undefined}
        value={clampedValue()}
        minValue={0}
        maxValue={1}
      />

      <Show when={showMessage() || (!isSmall() && bytesStr())}>
        <div class='gap-4 flex items-center justify-between'>
          <span
            class={cn('text-sm min-w-0 truncate', {
              'text-destructive': local.payload.status.type === 'failed',
              'text-muted-foreground': local.payload.status.type !== 'failed',
            })}
          >
            <Show when={showMessage()}>{message()}</Show>
          </span>
          <Show when={!isSmall() && bytesStr()}>
            <span class='text-sm text-muted-foreground shrink-0 tabular-nums'>
              {bytesStr()}
            </span>
          </Show>
        </div>
      </Show>
    </div>
  );
};
