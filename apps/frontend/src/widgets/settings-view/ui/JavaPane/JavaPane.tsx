import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

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
