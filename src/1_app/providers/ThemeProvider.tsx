import type { Component, JSX } from 'solid-js';

import { splitProps } from 'solid-js';

import { ThemeContext } from '@/shared/model';

import { useCreateThemeContext } from '../lib';

export type ThemeProviderProps = {
  children: JSX.Element;
  disableAnimationsAttribute: string;
  themeAttribute: string;
  themeStateKey: string;
  transparencyProperty: string;
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
