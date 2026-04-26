import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import ChangeLanguageEntry from './ChangeLanguageEntry';
import { DisableAnimationsEntry } from './DisableAnimationsEntry';
import { SelectActionOnInstanceLaunchEntry } from './SelectActionOnInstanceLaunchEntry';
import { SelectThemeEntry } from './SelectThemeEntry';
import SelectThemeForColorModeEntry from './SelectThemeForColorModeEntry';

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
      <SelectThemeEntry />
      <SelectThemeForColorModeEntry />
      <DisableAnimationsEntry />
      <ChangeLanguageEntry />
      <SelectActionOnInstanceLaunchEntry />
    </SettingsPane>
  );
};
