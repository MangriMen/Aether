import type { Component } from 'solid-js';

import { createEffect, createMemo, createSignal, Show } from 'solid-js';

import type { Option, ThemeConfig } from '@/shared/model';
import type { SelectRootProps } from '@/shared/ui';

import { THEMES, useThemeContext } from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Separator,
} from '@/shared/ui';

const THEME_OPTIONS: Option<ThemeConfig>[] = [
  ...THEMES.map((theme) => ({
    name: theme.name,
    value: theme.value,
  })),
  { name: 'System', value: 'system' },
];

export type SelectThemeProps = Pick<
  SelectRootProps<Option<ThemeConfig>, never>,
  'name' | 'id'
>;

export const SelectTheme: Component<SelectThemeProps> = (props) => {
  const [themeContext, { setTheme }] = useThemeContext();

  const [currentTheme, setCurrentTheme] = createSignal<ThemeConfig>(
    themeContext.rawTheme,
  );
  createEffect(() => {
    setCurrentTheme(themeContext.rawTheme);
  });

  const currentOption = createMemo(() =>
    THEME_OPTIONS.find((option) => option.value === currentTheme()),
  );

  const handleChangeTheme = (theme: Option<ThemeConfig> | null) => {
    if (!theme) {
      return;
    }

    setTheme(theme.value);
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
        <>
          <Show when={props.item.rawValue.value === 'system'}>
            <Separator class='my-1' />
          </Show>
          <SelectItem item={props.item}>{props.item.rawValue.name}</SelectItem>
        </>
      )}
      {...props}
    >
      <SelectTrigger>
        <SelectValue<Option<ThemeConfig>>>
          {(state) => state.selectedOption().name}
        </SelectValue>
      </SelectTrigger>
      <SelectContent />
    </Select>
  );
};
