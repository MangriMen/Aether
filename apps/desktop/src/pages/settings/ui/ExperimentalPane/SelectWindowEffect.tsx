import { useColorMode } from '@kobalte/core';
import { createMemo, type Component } from 'solid-js';

import {
  useAppSettings,
  useEditAppSettings,
  type WindowEffect,
} from '@/entities/settings';
import {
  isSystemTheme,
  useThemeContext,
  useTranslation,
  type Option,
} from '@/shared/model';
import { CombinedSelect, CombinedTooltip, SettingsEntry } from '@/shared/ui';

export type SelectWindowEffectProps = {
  class?: string;
};

type GeneralizedWindowEffect = 'off' | 'mica' | 'acrylic';

export const SELECT_WINDOW_EFFECT_OPTIONS: Option<GeneralizedWindowEffect>[] = [
  {
    name: 'off',
    value: 'off',
  },
  {
    name: 'mica',
    value: 'mica',
  },
  {
    name: 'acrylic',
    value: 'acrylic',
  },
] as const;

const WINDOW_EFFECT_TO_GENERALIZED_WINDOW_EFFECT: Record<
  WindowEffect,
  GeneralizedWindowEffect
> = {
  off: 'off',
  mica: 'mica',
  mica_light: 'mica',
  mica_dark: 'mica',
  acrylic: 'acrylic',
};

export const SelectWindowEffect: Component<SelectWindowEffectProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [theme] = useThemeContext();
  const { colorMode } = useColorMode();

  const appSettings = useAppSettings();
  const updateSettings = useEditAppSettings();

  const translatedWindowEffectOptions = createMemo(() =>
    SELECT_WINDOW_EFFECT_OPTIONS.map((option) => ({
      ...option,
      name: t(
        `settings.windowEffectVariant.${option.name as GeneralizedWindowEffect}`,
      ),
    })),
  );

  const currentWindowEffectOption = createMemo(() => {
    if (!appSettings.data?.windowEffect) {
      return translatedWindowEffectOptions()[0];
    }

    return (
      translatedWindowEffectOptions().find(
        (option) =>
          option.value ===
          WINDOW_EFFECT_TO_GENERALIZED_WINDOW_EFFECT[
            appSettings.data?.windowEffect
          ],
      ) ?? translatedWindowEffectOptions()[0]
    );
  });

  const handleSetWindowEffect = async (
    generalized_window_effect: Option<GeneralizedWindowEffect> | null,
  ) => {
    if (!generalized_window_effect) {
      return;
    }

    if (generalized_window_effect.value === 'mica') {
      updateSettings.mutateAsync({
        windowEffect: isSystemTheme(theme.rawTheme)
          ? 'mica'
          : `mica_${colorMode()}`,
      });
    } else {
      updateSettings.mutateAsync({
        windowEffect: generalized_window_effect.value,
      });
    }
  };

  const isDisabled = createMemo(() => !appSettings.data?.transparent);

  return (
    <SettingsEntry
      title={t('settings.windowEffect')}
      description={t('settings.windowEffectDescription')}
      {...props}
    >
      <CombinedTooltip
        label={isDisabled() ? t('settings.windowEffectDisabled') : undefined}
        disableTooltip={!isDisabled()}
        as={CombinedSelect}
        optionValue='value'
        optionTextValue='name'
        value={currentWindowEffectOption()}
        options={translatedWindowEffectOptions()}
        onChange={handleSetWindowEffect}
        disabled={isDisabled()}
      />
    </SettingsEntry>
  );
};
