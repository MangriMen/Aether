import { useColorMode } from '@kobalte/core';
import { type Component, createMemo } from 'solid-js';

import {
  isSystemTheme,
  type Option,
  useThemeContext,
  useTranslation,
} from '@/shared/model';
import { CombinedSelect, CombinedTooltip, SettingsEntry } from '@/shared/ui';

import type { WindowEffect } from '../../model';

import { useAppSettings, useUpdateAppSettings } from '../../api';

export type SelectWindowEffectProps = {
  class?: string;
};

type GeneralizedWindowEffect = 'acrylic' | 'mica' | 'off';

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
  acrylic: 'acrylic',
  mica: 'off',
  mica_dark: 'mica',
  mica_light: 'mica',
  off: 'off',
};

export const SelectWindowEffect: Component<SelectWindowEffectProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const [theme] = useThemeContext();
  const { colorMode } = useColorMode();

  const appSettings = useAppSettings();
  const updateSettings = useUpdateAppSettings();

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
    generalized_window_effect: null | Option<GeneralizedWindowEffect>,
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
      description={t('settings.windowEffectDescription')}
      title={t('settings.windowEffect')}
      {...props}
    >
      <CombinedTooltip
        as={CombinedSelect}
        disabled={isDisabled()}
        disableTooltip={!isDisabled()}
        label={isDisabled() ? t('settings.windowEffectDisabled') : undefined}
        onChange={handleSetWindowEffect}
        options={translatedWindowEffectOptions()}
        optionTextValue='name'
        optionValue='value'
        value={currentWindowEffectOption()}
      />
    </SettingsEntry>
  );
};
