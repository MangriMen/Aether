import { createEffect, on } from 'solid-js';

import { useAppSettings } from '@/entities/settings';
import { useThemeContext } from '@/shared/model';

export const useTransparencyUpdateListener = () => {
  const [_, { setTransparencyEnabled }] = useThemeContext();

  const appSettings = useAppSettings();

  createEffect(
    on([() => appSettings.data?.transparent], ([transparent]) => {
      setTransparencyEnabled(transparent ?? false);
    }),
  );
};
