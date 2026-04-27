import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { HooksSettings } from './HooksSettings';
import { JavaAndMemorySettings } from './JavaAndMemorySettings';
import { WindowSettings } from './WindowSettings';

export type DefaultInstanceSettingsPaneProps = ComponentProps<'div'>;

export const DefaultInstanceSettingsPane: Component<
  DefaultInstanceSettingsPaneProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.tab.defaultInstanceSettings')}
      {...others}
    >
      <WindowSettings class='rounded-md border bg-card/card p-4' />
      <JavaAndMemorySettings class='rounded-md border bg-card/card p-4' />
      <HooksSettings class='rounded-md border bg-card/card p-4' />
    </SettingsPane>
  );
};
