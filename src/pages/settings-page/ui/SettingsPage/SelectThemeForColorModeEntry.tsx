import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

// eslint-disable-next-line boundaries/element-types
import { useI18nContext, useThemeContext } from '@/app/model';

import { SettingsEntry } from '../SettingsEntry';

import SelectThemeForColorModeBlock from './SelectThemeForColorModeBlock';

export type SelectThemeForColorModeEntryProps = ComponentProps<'div'>;

const SelectThemeForColorModeEntry: Component<
  SelectThemeForColorModeEntryProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  const [{ t }] = useI18nContext();
  const [themeContext] = useThemeContext();

  const isSystemColorMode = createMemo(
    () => themeContext.rawTheme === 'system',
  );

  return (
    <SettingsEntry
      class={cn(local.class, { 'text-muted-foreground': !isSystemColorMode() })}
      title={t('systemColorMode')}
      description={t('systemColorModeDescription')}
      {...others}
    >
      <SelectThemeForColorModeBlock />
    </SettingsEntry>
  );
};

export default SelectThemeForColorModeEntry;
