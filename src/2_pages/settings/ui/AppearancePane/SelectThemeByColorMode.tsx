import type {
  ColorMode,
  ConfigColorMode,
  PolymorphicProps,
} from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import { createEffect, createMemo, createSignal } from 'solid-js';

import type { Option, ThemeConfig } from '@/shared/model';

import { THEME_BY_MODE, useThemeContext } from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

export type SelectThemeByColorModeProps<T extends ValidComponent = 'div'> = {
  colorMode: ColorMode;
} & PolymorphicProps<T, { disabled?: boolean; onChange?: never }>;

const THEME_BY_MODE_OPTIONS = Object.fromEntries(
  Object.entries(THEME_BY_MODE).map(([colorMode, themes]) => [
    colorMode as ColorMode,
    themes.map((theme) => ({
      name: theme.name,
      value: theme.value,
    })),
  ]),
);

const SelectThemeByColorMode: Component<SelectThemeByColorModeProps> = (
  props,
) => {
  const [themeContext, { setThemeByColorMode: setThemeForColorMode }] =
    useThemeContext();

  const options = createMemo(() => THEME_BY_MODE_OPTIONS[props.colorMode]);

  const [currentTheme, setCurrentTheme] = createSignal<ThemeConfig>('light');

  const currentOption = createMemo(() =>
    options().find((option) => option.value === currentTheme()),
  );

  const handleChangeTheme = (theme: null | Option<ThemeConfig>) => {
    if (!theme) {
      return;
    }

    setThemeForColorMode(props.colorMode, theme.value);
    setCurrentTheme(theme.value);
  };

  createEffect(() => {
    setCurrentTheme(themeContext[`${props.colorMode}Theme`]);
  });

  return (
    <Select
      class='w-32'
      defaultValue={currentOption()}
      itemComponent={(props) => (
        <SelectItem item={props.item}>{props.item.rawValue.name}</SelectItem>
      )}
      multiple={false}
      onChange={handleChangeTheme}
      options={options()}
      optionTextValue='name'
      optionValue='name'
      value={currentOption()}
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

export default SelectThemeByColorMode;
