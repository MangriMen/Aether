import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import type { ProgressEvent } from '@/entities/events';

import { useTranslation } from '@/shared/model';
import { Progress } from '@/shared/ui';

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
    }
  });

  return (
    <div class='flex flex-col gap-2' {...others}>
      <span class='font-bold'>{title()}</span>
      <Progress value={clampedValue()} minValue={0} maxValue={1} />
      <span class='text-muted-foreground'>
        {`${Math.round(clampedValue() * 100)}%`}
        &nbsp;
        {local.payload.message}
      </span>
    </div>
  );
};
