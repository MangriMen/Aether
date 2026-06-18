import type { Component, ParentProps } from 'solid-js';

import {
  COLOR_MODE_STORAGE_KEY,
  ColorModeProvider as KobalteColorModeProvider,
  ColorModeScript,
  createLocalStorageManager,
} from '@kobalte/core';

export type ColorModeObserverProps = ParentProps;

export const ColorModeProvider: Component<ColorModeObserverProps> = (props) => {
  const storageManager = createLocalStorageManager(COLOR_MODE_STORAGE_KEY);

  return (
    <>
      <ColorModeScript storageType={storageManager.type} />
      <KobalteColorModeProvider
        storageManager={storageManager}
        initialColorMode='dark'
      >
        {props.children}
      </KobalteColorModeProvider>
    </>
  );
};
