import { createMemo, type Component } from 'solid-js';

import type { CollapsibleSettingsEntryProps } from '@/shared/settings-layout';
import type { Option } from '@/shared/ui-kit';

import {
  MAX_CUSTOM_FONT_FAMILY_LENGTH,
  UI_FONTS,
  useThemeContext,
  type UiFont,
} from '@/features/appearance';
import { useTranslation } from '@/shared/i18n';
import {
  CollapsibleSettingsEntry,
  SettingsEntry,
} from '@/shared/settings-layout';
import { CombinedTextField, Separator } from '@/shared/ui-kit';

import SelectUiFont from './SelectUiFont';

export type UiFontEntryProps = CollapsibleSettingsEntryProps;

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
    <CollapsibleSettingsEntry
      title={t('settings.uiFont')}
      description={t('settings.uiFontDescription')}
      collapsibleContent={
        <>
          <Separator />
          <SettingsEntry
            class='px-4'
            title={t('settings.uiFontCustomName')}
            description={t('settings.uiFontCustomDescription')}
          >
            <CombinedTextField
              class='w-48'
              disabled={themeContext.uiFont !== UI_FONTS.Custom}
              value={themeContext.customFontFamily}
              onChange={setCustomFontFamily}
              inputProps={{
                type: 'text',
                maxLength: MAX_CUSTOM_FONT_FAMILY_LENGTH,
                placeholder: t('settings.uiFontCustomPlaceholder'),
              }}
            />
          </SettingsEntry>
        </>
      }
      {...props}
    >
      <SelectUiFont
        class='w-40'
        options={options()}
        value={currentOption()}
        onChange={handleChange}
      />
    </CollapsibleSettingsEntry>
  );
};
