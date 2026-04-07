import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type FieldLabelProps = ComponentProps<'span'>;

export const FieldLabel: Component<FieldLabelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return <span class={cn('font-medium', local.class)} {...others} />;
};
