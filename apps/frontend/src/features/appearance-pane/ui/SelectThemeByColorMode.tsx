import type {
  ColorMode,
  ConfigColorMode,
  PolymorphicProps,
} from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import { createEffect, createMemo, createSignal } from 'solid-js';

import type { ThemeConfig } from '@/features/appearance';
import type { Option } from '@/shared/ui-kit';

import {
  getThemeLabel,
  THEME_BY_MODE,
  useThemeContext,
} from '@/features/appearance';
import { useTranslation } from '@/shared/i18n';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui-kit';

type SelectThemeByColorModeProps<T extends ValidComponent = 'div'> =
  PolymorphicProps<T, { disabled?: boolean; onChange?: never }> & {
    colorMode: ColorMode;
  };

const SelectThemeByColorMode: Component<SelectThemeByColorModeProps> = (
  props,
) => {
  const [themeContext, { setThemeByColorMode: setThemeForColorMode }] =
    useThemeContext();

  const [{ t }] = useTranslation();

  const options = createMemo<Option<ThemeConfig>[]>(() =>
    (THEME_BY_MODE[props.colorMode] ?? []).map((theme) => ({
      name: getThemeLabel(theme, t),
      value: theme.value,
    })),
  );

  const [currentTheme, setCurrentTheme] = createSignal<ThemeConfig>('light');

  const currentOption = createMemo(() =>
    options().find((option) => option.value === currentTheme()),
  );

  const handleChangeTheme = (theme: Option<ThemeConfig> | null) => {
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
      multiple={false}
      options={options()}
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

export default SelectThemeByColorMode;
