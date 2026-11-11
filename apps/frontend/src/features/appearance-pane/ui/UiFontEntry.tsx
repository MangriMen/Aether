import { createMemo, Show, type Component } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/settings-layout';
import type { Option } from '@/shared/ui-kit';

import {
  MAX_CUSTOM_FONT_FAMILY_LENGTH,
  UI_FONTS,
  useThemeContext,
  type UiFont,
} from '@/features/appearance';
import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';
import { CombinedTextField } from '@/shared/ui-kit';

import SelectUiFont from './SelectUiFont';

export type UiFontEntryProps = SettingsEntryProps;

export const UiFontEntry: Component<UiFontEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  const [themeContext, { setUiFont, setCustomFontFamily }] = useThemeContext();

  const options = createMemo<Option<UiFont>[]>(() => [
    { name: t('settings.uiFontSystem') ?? '', value: UI_FONTS.System },
    { name: 'Inter', value: UI_FONTS.Inter },
    { name: t('settings.uiFontCustom') ?? '', value: UI_FONTS.Custom },
  ]);

  const currentOption = createMemo(() =>
    options().find((option) => option.value === themeContext.uiFont),
  );

  const handleChange = (option: Option<UiFont> | null) => {
    if (option) {
      setUiFont(option.value);
    }
  };

  return (
    <>
      <SettingsEntry
        title={t('settings.uiFont')}
        description={t('settings.uiFontDescription')}
        {...props}
      >
        <SelectUiFont
          class='w-40'
          options={options()}
          value={currentOption()}
          onChange={handleChange}
        />
      </SettingsEntry>
      <Show when={themeContext.uiFont === UI_FONTS.Custom}>
        <SettingsEntry
          title={t('settings.uiFontCustomName')}
          description={t('settings.uiFontCustomDescription')}
          {...props}
        >
          <CombinedTextField
            class='w-48'
            value={themeContext.customFontFamily}
            onChange={setCustomFontFamily}
            inputProps={{
              type: 'text',
              maxLength: MAX_CUSTOM_FONT_FAMILY_LENGTH,
              placeholder: t('settings.uiFontCustomPlaceholder'),
            }}
          />
        </SettingsEntry>
      </Show>
    </>
  );
};
