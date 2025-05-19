import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import type { CheckboxRootProps } from '@/shared/ui';
import { Checkbox } from '@/shared/ui';

import { useTranslate } from '@/shared/model';

export type IncludeSnapshotsCheckboxProps<
  T extends ValidComponent = 'checkbox',
> = PolymorphicProps<T, CheckboxRootProps<T>> & {
  show?: boolean;
};

export const IncludeSnapshotsCheckbox: Component<
  IncludeSnapshotsCheckboxProps
> = (props) => {
  const [local, others] = splitProps(props, ['show', 'class']);

  const [{ t }] = useTranslate();

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
      <Checkbox
        class='text-sm font-semibold'
        label={t('createInstance.includeSnapshots')}
        {...others}
      />
    </div>
  );
};
