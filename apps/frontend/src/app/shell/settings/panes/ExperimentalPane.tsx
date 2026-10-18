import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/settings-layout';

import { useTranslation } from '@/shared/i18n';
import { SettingsPane } from '@/shared/settings-layout';

import { FeatureFlagsEntry } from './FeatureFlagsEntry';

export type ExperimentalPaneProps = SettingsPaneProps;

export const ExperimentalPane: Component<ExperimentalPaneProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsPane label={t('settings.tab.experimental')} {...props}>
      <FeatureFlagsEntry variant='card' />
    </SettingsPane>
  );
};
