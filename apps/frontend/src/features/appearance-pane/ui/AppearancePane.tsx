import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/settings-layout';

import { useTranslation } from '@/shared/i18n';
import { SettingsPane } from '@/shared/settings-layout';

import { ChangeLanguageEntry } from './ChangeLanguageEntry';
import { DisableAnimationsEntry } from './DisableAnimationsEntry';
import { SelectActionOnInstanceLaunchEntry } from './SelectActionOnInstanceLaunchEntry';
import { ThemeEntry } from './ThemeEntry';
import { WindowTransparencyEntry } from './WindowTransparencyEntry';

export type AppearancePaneProps = SettingsPaneProps;

export const AppearancePane: Component<AppearancePaneProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsPane label={t('settings.tab.appearance')} {...props}>
      <ThemeEntry variant='card' />
      <WindowTransparencyEntry variant='card' />
      <DisableAnimationsEntry variant='card' />
      <SelectActionOnInstanceLaunchEntry variant='card' />
      <ChangeLanguageEntry variant='card' />
    </SettingsPane>
  );
};
