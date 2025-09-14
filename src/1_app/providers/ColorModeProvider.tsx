import type { RouteSectionProps } from '@solidjs/router';
import type { Component } from 'solid-js';

import {
  COLOR_MODE_STORAGE_KEY,
  ColorModeScript,
  createLocalStorageManager,
  ColorModeProvider as KobalteColorModeProvider,
} from '@kobalte/core';

export type ColorModeObserverProps = RouteSectionProps;

export const ColorModeProvider: Component<ColorModeObserverProps> = (props) => {
  const storageManager = createLocalStorageManager(COLOR_MODE_STORAGE_KEY);

  return (
    <>
      <ColorModeScript storageType={storageManager.type} />
      <KobalteColorModeProvider
        initialColorMode='dark'
        storageManager={storageManager}
      >
        {props.children}
      </KobalteColorModeProvider>
    </>
  );
};
