import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import type { ProgressEvent } from '@/entities/events';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, Progress } from '@/shared/ui';

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

  const percentStr = createMemo(() => `${Math.round(clampedValue() * 100)}%`);

  return (
    <div class='flex flex-col gap-2' {...others}>
      <div class='flex items-end justify-between gap-4'>
        <CombinedTooltip label={title()} as='span' class='truncate font-bold'>
          {title()}
        </CombinedTooltip>
        <span class='shrink-0 tabular-nums text-muted-foreground'>
          {percentStr()}
        </span>
      </div>

      <Progress value={clampedValue()} minValue={0} maxValue={1} />

      <span class='truncate text-sm text-muted-foreground'>{message()}</span>
    </div>
  );
};
