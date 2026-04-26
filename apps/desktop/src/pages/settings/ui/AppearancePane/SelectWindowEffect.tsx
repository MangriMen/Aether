import { useColorMode } from '@kobalte/core';
import { createMemo } from 'solid-js';

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
import { CombinedSelect, CombinedTooltip } from '@/shared/ui';

export type WindowEffectEntryProps = {
  class?: string;
};

type GeneralizedWindowEffect = 'off' | 'mica' | 'acrylic';

export const SELECT_WINDOW_EFFECT_OPTIONS: Option<
  GeneralizedWindowEffect,
  GeneralizedWindowEffect
>[] = [
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

const WINDOW_EFFECT_MAP: Record<WindowEffect, GeneralizedWindowEffect> = {
  off: 'off',
  mica: 'mica',
  mica_light: 'mica',
  mica_dark: 'mica',
  acrylic: 'acrylic',
};

export const SelectWindowEffect = () => {
  const [{ t }] = useTranslation();
  const [theme] = useThemeContext();
  const { colorMode } = useColorMode();

  const appSettings = useAppSettings();
  const updateSettings = useEditAppSettings();

  const isDisabled = createMemo(() => !appSettings.data?.transparent);

  const translatedOptions = createMemo(() =>
    SELECT_WINDOW_EFFECT_OPTIONS.map((option) => ({
      ...option,
      name: t(`settings.windowEffectVariant.${option.name}`),
    })),
  );

  const currentOption = createMemo(() => {
    const effect = appSettings.data?.windowEffect;
    const generalizedValue = effect ? WINDOW_EFFECT_MAP[effect] : 'off';

    return (
      translatedOptions().find((opt) => opt.value === generalizedValue) ??
      translatedOptions()[0]
    );
  });

  const handleSetWindowEffect = async (
    option: Option<GeneralizedWindowEffect> | null,
  ) => {
    if (!option) {
      return;
    }

    let newValue: WindowEffect = option.value;

    if (option.value === 'mica') {
      newValue = isSystemTheme(theme.rawTheme) ? 'mica' : `mica_${colorMode()}`;
    }

    updateSettings.mutateAsync({
      windowEffect: newValue,
    });
  };

  return (
    <CombinedTooltip
      label={isDisabled() ? t('settings.windowEffectDisabled') : undefined}
      disableTooltip={!isDisabled()}
      as={CombinedSelect}
      optionValue='value'
      optionTextValue='name'
      value={currentOption()}
      options={translatedOptions()}
      onChange={handleSetWindowEffect}
      disabled={isDisabled()}
    />
  );
};
