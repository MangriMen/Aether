import { Component } from 'solid-js';

import { AppVersion } from '@/entities/settings';

import { SelectColorMode } from '@/features/select-color-mode';

import { SettingsEntry } from '../SettingsEntry';
import { SettingsPane } from '../SettingsPane';

import { SettingsPageProps } from './types';

export const SettingsPage: Component<SettingsPageProps> = (props) => {
  return (
    <div class='flex size-full flex-col p-4' {...props}>
      <SettingsPane title='Launcher'>
        <SettingsEntry
          title='Color theme'
          description='Change global launcher theme'
        >
          <SelectColorMode />
        </SettingsEntry>
      </SettingsPane>
      <AppVersion class='mt-auto' />
    </div>
  );
};
