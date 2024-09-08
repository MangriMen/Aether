import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { FieldLabelProps } from '.';

export const FieldLabel: Component<FieldLabelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return <span class={cn('text-sm font-medium', local.class)} {...others} />;
};
