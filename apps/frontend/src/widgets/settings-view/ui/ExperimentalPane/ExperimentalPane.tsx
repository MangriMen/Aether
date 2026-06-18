import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { FeatureFlagsEntry } from './FeatureFlagsEntry';
import { GoToPlaygroundEntry } from './GoToPlaygroundEntry';

export type ExperimentalPaneProps = SettingsPaneProps;

export const ExperimentalPane: Component<ExperimentalPaneProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsPane label={t('settings.tab.experimental')} {...props}>
      <GoToPlaygroundEntry variant='card' />
      <FeatureFlagsEntry variant='card' />
    </SettingsPane>
  );
};
