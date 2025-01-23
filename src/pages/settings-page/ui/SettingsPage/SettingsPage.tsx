import { Component, createMemo } from 'solid-js';

import { cn } from '@/shared/lib';

import { AppVersion } from '@/entities/settings';

import { SelectTheme } from '@/features/select-color-mode';

// eslint-disable-next-line boundaries/element-types
import { useThemeContext } from '@/app/model';

import { SettingsEntry } from '../SettingsEntry';
import { SettingsPane } from '../SettingsPane';

import SelectThemeForColorModeBlock from './SelectThemeForColorModeBlock';
import { SettingsPageProps } from './types';

export const SettingsPage: Component<SettingsPageProps> = (props) => {
  const [themeContext] = useThemeContext();

  const isSystemColorMode = createMemo(
    () => themeContext.rawTheme === 'system',
  );
  return (
    <div class='flex size-full flex-col p-4' {...props}>
      <SettingsPane class='mx-auto w-full max-w-screen-lg' title='Launcher'>
        <SettingsEntry
          title='Color theme'
          description='Select your referred color theme'
        >
          <SelectTheme />
        </SettingsEntry>
        <SettingsEntry
          class={cn({ 'text-muted-foreground': !isSystemColorMode() })}
          title='System color mode'
          description={
            'Automatically adapt to your system\'s color mode.\nThis option is applied only when the color theme is set to "System"'
          }
        >
          <SelectThemeForColorModeBlock />
        </SettingsEntry>
      </SettingsPane>
      <AppVersion class='mt-auto' />
    </div>
  );
};
