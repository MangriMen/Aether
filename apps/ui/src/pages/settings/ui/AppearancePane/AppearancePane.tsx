import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { ChangeLanguageEntry } from './ChangeLanguageEntry';
import { DisableAnimationsEntry } from './DisableAnimationsEntry';
import { SelectActionOnInstanceLaunchEntry } from './SelectActionOnInstanceLaunchEntry';
import { ThemeEntry } from './ThemeEntry';
import { WindowTransparencyEntry } from './WindowTransparencyEntry';

export type AppearancePaneProps = ComponentProps<'div'>;

export const AppearancePane: Component<AppearancePaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.tab.appearance')}
      {...others}
    >
      <ThemeEntry variant='card' />
      <WindowTransparencyEntry variant='card' />
      <DisableAnimationsEntry variant='card' />
      <SelectActionOnInstanceLaunchEntry variant='card' />
      <ChangeLanguageEntry variant='card' />
    </SettingsPane>
  );
};
