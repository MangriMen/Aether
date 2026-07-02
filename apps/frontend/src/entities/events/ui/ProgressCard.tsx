import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, Progress } from '@/shared/ui';

import type { ProgressEvent } from '..';

export type ProgressCardProps = ComponentProps<'div'> & {
  payload: ProgressEvent;
};

export const ProgressCard: Component<ProgressCardProps> = (props) => {
  const [local, others] = splitProps(props, ['payload', 'class']);

  const [{ t }] = useTranslation();

  const clampedValue = createMemo(() => local.payload.fraction ?? 1);

  const title = createMemo(() => {
    switch (local.payload.event.type) {
      case 'java_download':
        return t('events.javaDownloading', {
          version: local.payload.event.version,
        });
      case 'minecraft_download':
        return local.payload.event.instance_name;
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

  const message = createMemo(() => {
    // eslint-disable-next-line sonarjs/no-small-switch
    switch (local.payload.event.type) {
      case 'launcher_update':
        return t(`update.events.phase.${local.payload.event.phase}`);
      default:
        return local.payload.message;
    }
  });

  const isError = createMemo(
    () =>
      local.payload.event.type === 'launcher_update' &&
      local.payload.event.phase === 'error',
  );

  const percentStr = createMemo(() => `${Math.round(clampedValue() * 100)}%`);

  return (
    <div class='gap-2 flex flex-col' {...others}>
      <div class='gap-4 flex items-end justify-between'>
        <CombinedTooltip label={title()} as='span' class='font-bold truncate'>
          {title()}
        </CombinedTooltip>
        <span class='text-muted-foreground shrink-0 tabular-nums'>
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
