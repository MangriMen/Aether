import { createEffect, on } from 'solid-js';

import { useAppSettings } from '@/entities/settings';
import { useThemeContext } from '@/shared/model';

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
