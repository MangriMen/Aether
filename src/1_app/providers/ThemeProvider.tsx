import type { Component, JSX } from 'solid-js';
import { splitProps } from 'solid-js';

import { ThemeContext } from '@/shared/model';
import { useCreateThemeContext } from '../lib';

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

  const context = useCreateThemeContext(
    () => local.themeStateKey,
    () => local.themeAttribute,
    () => local.disableAnimationsAttribute,
    () => local.transparencyProperty,
  );

  return <ThemeContext.Provider value={context} {...others} />;
};
