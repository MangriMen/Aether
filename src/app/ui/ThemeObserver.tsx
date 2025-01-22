import { useColorMode } from '@kobalte/core';
import { Component, JSX, onMount } from 'solid-js';

import { getTheme, setTheme } from '@/shared/model';

export type ThemeObserverProps = { children?: JSX.Element };

const ThemeObserver: Component<ThemeObserverProps> = (props) => {
  const colorModeContext = useColorMode();

  onMount(() => {
    const theme = getTheme();

    if (!theme) {
      const colorMode = colorModeContext.colorMode();

      setTheme(colorMode === 'light' ? 'aether-light' : 'aether-dark');
      return;
    }

    setTheme(theme);
  });

  return <>{props.children}</>;
};

export default ThemeObserver;
