import { Component } from 'solid-js';

import { SelectColorMode } from '@/features/select-color-mode';

import { SettingsEntry } from '../SettingsEntry';
import { SettingsPane } from '../SettingsPane';

import { SettingsPageProps } from './types';

export const SettingsPage: Component<SettingsPageProps> = (props) => {
  return (
    <div class='size-full p-4' {...props}>
      <SettingsPane title='Launcher'>
        <SettingsEntry
          title='Color theme'
          description='Change global launcher theme'
        >
          <SelectColorMode id='color-mode' />
        </SettingsEntry>
      </SettingsPane>
    </div>
  );
};
