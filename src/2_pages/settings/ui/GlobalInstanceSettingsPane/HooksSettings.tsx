import type { Component, ComponentProps } from 'solid-js';

export type HooksSettingsProps = ComponentProps<'div'>;

export const HooksSettings: Component<HooksSettingsProps> = (props) => {
  return <div {...props} />;
};
