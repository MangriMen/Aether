import type { Component, JSX } from 'solid-js';

import { splitProps } from 'solid-js';

import { createThemeContext } from '../lib/createThemeContext';
import { ThemeContext } from '../model/themeContext';

export type ThemeProviderProps = {
  themeStateKey: string;
  themeAttribute: string;
  disableAnimationsAttribute: string;
  transparencyProperty: string;
  children: JSX.Element;
};

export const ThemeProvider: Component<ThemeProviderProps> = (props) => {
  const [local, others] = splitProps(props, [
    'themeStateKey',
    'themeAttribute',
    'disableAnimationsAttribute',
    'transparencyProperty',
  ]);

  const context = createThemeContext(
    () => local.themeStateKey,
    () => local.themeAttribute,
    () => local.disableAnimationsAttribute,
    () => local.transparencyProperty,
  );

  return <ThemeContext.Provider value={context} {...others} />;
};
