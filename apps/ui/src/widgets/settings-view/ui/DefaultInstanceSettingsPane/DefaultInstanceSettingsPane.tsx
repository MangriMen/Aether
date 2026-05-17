import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { HooksSettings } from './HooksSettings';
import { JavaAndMemorySettings } from './JavaAndMemorySettings';
import { WindowSettings } from './WindowSettings';

export type DefaultInstanceSettingsPaneProps = SettingsPaneProps;

export const DefaultInstanceSettingsPane: Component<
  DefaultInstanceSettingsPaneProps
> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsPane label={t('settings.tab.defaultInstanceSettings')} {...props}>
      <WindowSettings class='rounded-md border bg-card/card p-4' />
      <JavaAndMemorySettings class='rounded-md border bg-card/card p-4' />
      <HooksSettings class='rounded-md border bg-card/card p-4' />
    </SettingsPane>
  );
};
