import type { ComponentProps, JSX } from 'solid-js';

export type SettingsEntryProps = ComponentProps<'div'> & {
  title?: string | JSX.Element;
  description?: string | JSX.Element;
};
