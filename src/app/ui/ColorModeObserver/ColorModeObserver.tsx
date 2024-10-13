import {
  COLOR_MODE_STORAGE_KEY,
  ColorModeProvider,
  ColorModeScript,
  createLocalStorageManager,
} from '@kobalte/core';
import { Component } from 'solid-js';

import { ColorModeObserverProps } from './types';

export const ColorModeObserver: Component<ColorModeObserverProps> = (props) => {
  const storageManager = createLocalStorageManager(COLOR_MODE_STORAGE_KEY);

  return (
    <>
      <ColorModeScript storageType={storageManager.type} />
      <ColorModeProvider
        storageManager={storageManager}
        initialColorMode='dark'
      >
        {props.children}
      </ColorModeProvider>
    </>
  );
};
