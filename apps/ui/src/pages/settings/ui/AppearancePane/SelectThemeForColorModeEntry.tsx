import type { Component } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation, useThemeContext } from '@/shared/model';
import { SettingsEntry } from '@/shared/ui';

import SelectThemeForColorMode from './SelectThemeForColorMode';

export type SelectThemeForColorModeEntryProps = SettingsEntryProps;

export const SelectThemeForColorModeEntry: Component<
  SelectThemeForColorModeEntryProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  const [{ t }] = useTranslation();
  const [themeContext] = useThemeContext();

  const isSystemColorMode = createMemo(
    () => themeContext.rawTheme === 'system',
  );

  return (
    <SettingsEntry
      class={cn(local.class, {
        'text-muted-foreground': !isSystemColorMode(),
      })}
      title={t('settings.systemColorMode')}
      description={t('settings.systemColorModeDescription')}
      {...others}
    >
      <SelectThemeForColorMode />
    </SettingsEntry>
  );
};
