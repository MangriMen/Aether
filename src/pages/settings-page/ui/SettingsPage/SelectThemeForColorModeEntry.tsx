import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

// eslint-disable-next-line boundaries/element-types
import { useThemeContext } from '@/app/model';

import { SettingsEntry } from '../SettingsEntry';

import SelectThemeForColorModeBlock from './SelectThemeForColorModeBlock';

export type SelectThemeForColorModeEntryProps = ComponentProps<'div'>;

const SelectThemeForColorModeEntry: Component<
  SelectThemeForColorModeEntryProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  const [themeContext] = useThemeContext();

  const isSystemColorMode = createMemo(
    () => themeContext.rawTheme === 'system',
  );

  return (
    <SettingsEntry
      class={cn(local.class, { 'text-muted-foreground': !isSystemColorMode() })}
      title='System color mode'
      description={
        'Automatically adapt to your system\'s color mode.\nThis option is applied only when the color theme is set to "System"'
      }
      {...others}
    >
      <SelectThemeForColorModeBlock />
    </SettingsEntry>
  );
};

export default SelectThemeForColorModeEntry;
