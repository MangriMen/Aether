import { ConfigColorMode, useColorMode } from '@kobalte/core';
import { Component, createMemo, createSignal } from 'solid-js';

import {
  getTheme,
  Option,
  setTheme,
  Theme,
  THEME_TO_MODE,
} from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import { SelectColorModeProps } from './types';

const THEME_OPTIONS: Option<Theme>[] = [
  { name: 'Light', value: 'light' },
  { name: 'Aether Light', value: 'aether-light' },
  { name: 'Dark', value: 'dark' },
  { name: 'Aether Dark', value: 'aether-dark' },
  { name: 'System', value: 'system' },
];

export const SelectColorMode: Component<SelectColorModeProps> = (props) => {
  const colorModeContext = useColorMode();

  const [currentTheme, setCurrentTheme] = createSignal<Theme>(
    getTheme() ?? colorModeContext.colorMode(),
  );

  const currentOption = createMemo(() =>
    THEME_OPTIONS.find((option) => option.value === currentTheme()),
  );

  const handleChangeTheme = (theme: Option<Theme> | null) => {
    if (!theme) {
      return;
    }

    handleChangeColorMode(THEME_TO_MODE[theme.value]);

    setCurrentTheme(theme.value);
    setTheme(theme.value);
  };

  const handleChangeColorMode = (colorMode: ConfigColorMode) => {
    colorModeContext.setColorMode(colorMode);
  };

  return (
    <Select
      class='w-32'
      options={THEME_OPTIONS}
      optionValue='name'
      optionTextValue='name'
      defaultValue={currentOption()}
      value={currentOption()}
      onChange={handleChangeTheme}
      itemComponent={(props) => (
        <SelectItem item={props.item}>{props.item.rawValue.name}</SelectItem>
      )}
      {...props}
    >
      <SelectTrigger>
        <SelectValue<Option<ConfigColorMode>>>
          {(state) => state.selectedOption().name}
        </SelectValue>
      </SelectTrigger>
      <SelectContent />
    </Select>
  );
};
