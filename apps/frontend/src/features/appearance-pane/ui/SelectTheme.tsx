import type { Component, ComponentProps } from 'solid-js';

import { createEffect, createMemo, createSignal, Show } from 'solid-js';

import type { ThemeConfig } from '@/features/appearance';
import type { SelectRootProps, Option } from '@/shared/ui-kit';

import { getThemeLabel, THEMES, useThemeContext } from '@/features/appearance';
import { useTranslation } from '@/shared/i18n';
import { useSettingsEntryLabel } from '@/shared/settings-layout';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Separator,
} from '@/shared/ui-kit';

export type SelectThemeProps = Pick<
  SelectRootProps<Option<ThemeConfig>, never>,
  'name' | 'id'
>;

export const SelectTheme: Component<
  Omit<ComponentProps<'div'>, 'onChange'> & SelectThemeProps
> = (props) => {
  const [{ t }] = useTranslation();
  const [themeContext, { setTheme }] = useThemeContext();

  const themeOptions = createMemo<Option<ThemeConfig>[]>(() => [
    ...THEMES.map((theme) => ({
      name: getThemeLabel(theme, t),
      value: theme.value,
    })),
    { name: t('settings.themes.system') ?? 'system', value: 'system' },
  ]);

  const [currentTheme, setCurrentTheme] = createSignal<ThemeConfig>(
    themeContext.rawTheme,
  );
  createEffect(() => {
    setCurrentTheme(themeContext.rawTheme);
  });

  const currentOption = createMemo(() =>
    themeOptions().find((option) => option.value === currentTheme()),
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
      options={themeOptions()}
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
      <SelectTrigger {...useSettingsEntryLabel()}>
        <SelectValue<Option<ThemeConfig>>>
          {(state) => state.selectedOption().name}
        </SelectValue>
      </SelectTrigger>
      <SelectContent />
    </Select>
  );
};
