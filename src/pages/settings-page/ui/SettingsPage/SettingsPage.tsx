import type { RouteSectionProps } from '@solidjs/router';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { AppVersion } from '@/entities/settings';

import { SelectTheme } from '@/features/select-color-mode';

import { SettingsEntry } from '../SettingsEntry';
import { SettingsPane } from '../SettingsPane';

import SelectThemeForColorModeEntry from './SelectThemeForColorModeEntry';
import UpdateAppEntry from './UpdateAppEntry';

export type SettingsPageProps = ComponentProps<'div'> & RouteSectionProps;

export const SettingsPage: Component<SettingsPageProps> = (props) => {
  const [_, others] = splitProps(props, ['params', 'location', 'data']);
  return (
    <div class='flex size-full flex-col gap-4 p-4' {...others}>
      <SettingsPane class='container max-w-screen-lg' title='Launcher'>
        <SettingsEntry
          title='Color theme'
          description='Select your referred color theme'
        >
          <SelectTheme />
        </SettingsEntry>
        <SelectThemeForColorModeEntry />
      </SettingsPane>
      <SettingsPane class='container max-w-screen-lg' title='Update'>
        <UpdateAppEntry />
      </SettingsPane>
      <AppVersion class='mt-auto' />
    </div>
  );
};
