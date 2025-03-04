import type { RouteSectionProps } from '@solidjs/router';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useTranslate } from '@/shared/model';

import { SettingsEntry } from '../SettingsEntry';

import ChangeLanguageEntry from './ChangeLanguageEntry';
import SelectThemeForColorModeEntry from './SelectThemeForColorModeEntry';
import UpdateAppEntry from './UpdateAppEntry';
import { SettingsPane } from '../SettingsPane';
import { AppVersion } from '../AppVersion';
import { SelectTheme } from '../SelectTheme';
import { SelectActionOnInstanceLaunchEntry } from './SelectActionOnInstanceLaunchEntry';
import { PluginsPane } from '../PluginSettings/PluginsPane';

export type SettingsPageProps = ComponentProps<'div'> & RouteSectionProps;

export const SettingsPage: Component<SettingsPageProps> = (props) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);
  const [{ t }] = useTranslate();

  return (
    <div class='flex size-full flex-col gap-4 overflow-y-auto p-4' {...others}>
      <SettingsPane
        class='container max-w-screen-lg'
        label={t('settings.launcher')}
      >
        <SettingsEntry
          title={t('settings.colorScheme')}
          description={t('settings.colorSchemeDescription')}
        >
          <SelectTheme />
        </SettingsEntry>
        <SelectThemeForColorModeEntry />
        <ChangeLanguageEntry />
        <SelectActionOnInstanceLaunchEntry />
      </SettingsPane>
      <SettingsPane
        class='container max-w-screen-lg'
        label={t('settings.update')}
      >
        <UpdateAppEntry />
      </SettingsPane>
      <PluginsPane />
      <AppVersion class='mt-auto' />
    </div>
  );
};
