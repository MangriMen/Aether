import type { Component } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { Progress } from '@/shared/ui';

import { LoadingBarTypeEnum } from '@/entities/events';

import type { EventCardProps } from './types';
import { useTranslation } from '@/shared/model';

export const EventCard: Component<EventCardProps> = (props) => {
  const [local, others] = splitProps(props, ['payload', 'class']);

  const [{ t }] = useTranslation();

  const clampedValue = createMemo(() => local.payload.fraction ?? 1);

  const title = createMemo(() => {
    switch (local.payload.event.type) {
      case LoadingBarTypeEnum.JavaDownload:
        return t('events.javaDownloading', {
          version: local.payload.event.version,
        });
      case LoadingBarTypeEnum.MinecraftDownload:
        return local.payload.event.instance_name;
      case LoadingBarTypeEnum.PluginDownload:
        return `[Plugin] ${local.payload.event.plugin_name}`;
      case LoadingBarTypeEnum.LauncherUpdate:
        return t('events.launcherUpdating', {
          version: local.payload.event.version,
        });
    }
  });

  return (
    <div
      class='flex flex-col gap-2 rounded-md border bg-background p-3'
      {...others}
    >
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
