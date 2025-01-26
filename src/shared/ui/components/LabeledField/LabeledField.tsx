import type { Component } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { FieldLabel } from '../FieldLabel';

import type { FieldProps } from '.';

export const LabeledField: Component<FieldProps> = (props) => {
  const [local, others] = splitProps(props, ['label', 'class', 'children']);

  const labelElement = createMemo(() =>
    typeof local.label === 'string' ? (
      <FieldLabel>{local.label}</FieldLabel>
    ) : (
      local.label
    ),
  );

  return (
    <div class={cn('flex flex-col gap-2 text-sm', local.class)} {...others}>
      {labelElement()}
      {local.children}
    </div>
  );
};
