import { createEffect, on } from 'solid-js';

import { useAppSettings } from '@/features/settings';

import { useThemeContext } from '../model/themeContext';

export const useTransparencyUpdateListener = () => {
  const [_, { setIsActualTransparent, setTransparencyEnabled }] =
    useThemeContext();

  const appSettings = useAppSettings();

  createEffect(
    on(
      [
        () => appSettings.data?.isActualTransparent,
        () => appSettings.data?.transparent,
      ],
      ([isActualTransparent, transparent]) => {
        setIsActualTransparent(isActualTransparent ?? false);
        setTransparencyEnabled(transparent ?? false);
      },
    ),
  );
};
