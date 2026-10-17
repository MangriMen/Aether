import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/settings-layout';

import { useTranslation } from '@/shared/i18n';
import { SettingsPane } from '@/shared/settings-layout';

import { JavaVersionsTable } from './JavaVersionsTable';

export type JavaPaneProps = SettingsPaneProps;

export const JavaPane: Component<JavaPaneProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsPane label={t('settings.tab.java')} {...props}>
      <JavaVersionsTable />
    </SettingsPane>
  );
};
