import { PolymorphicProps } from '@kobalte/core';
import {
  Component,
  createUniqueId,
  splitProps,
  ValidComponent,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { Checkbox, CheckboxRootProps, Label } from '@/shared/ui';

export type IncludeSnapshotsCheckboxProps<
  T extends ValidComponent = 'checkbox',
> = PolymorphicProps<T, CheckboxRootProps<T>> & {
  show?: boolean;
};

export const IncludeSnapshotsCheckbox: Component<
  IncludeSnapshotsCheckboxProps
> = (props) => {
  const [local, others] = splitProps(props, ['show', 'class']);

  const id = createUniqueId();

  return (
    <div
      class={cn(
        'flex min-w-max items-center gap-2 animate-in fade-in-0 duration-300',
        {
          'invisible animate-out fade-out-0': !local.show,
        },
        local.class,
      )}
    >
      <Checkbox id={id} {...others} />
      <Label for={`${id}-input`}>Include snapshots</Label>
    </div>
  );
};
