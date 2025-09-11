import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Separator, SettingsPane } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import { HooksSettings } from './HooksSettings';
import { WindowSettings } from './WindowSettings';
import { JavaAndMemorySettings } from './JavaAndMemorySettings';

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
      <WindowSettings />
      <Separator />
      <JavaAndMemorySettings />
      <Separator />
      <HooksSettings />
    </SettingsPane>
  );
};
