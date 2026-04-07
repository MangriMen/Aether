import type { Component, ComponentProps } from 'solid-js';

export type TabConfig<
  TValue extends string,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  TTabProps extends Record<string, any> = object,
> = {
  value: TValue;
  label: string;
  icon?: Component<ComponentProps<'svg'>>;
  component: Component<TTabProps>;
};
