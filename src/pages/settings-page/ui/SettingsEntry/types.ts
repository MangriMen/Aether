import { ComponentProps } from 'solid-js';

export type SettingsEntryProps = ComponentProps<'div'> & {
  title?: string;
  description?: string;
};
