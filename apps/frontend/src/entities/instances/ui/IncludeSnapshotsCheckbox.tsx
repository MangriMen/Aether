import type { PolymorphicProps } from '@kobalte/core';
import type { Component, ValidComponent } from 'solid-js';

import { splitProps } from 'solid-js';

import type { CheckboxRootProps } from '@/shared/ui';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Checkbox } from '@/shared/ui';

export type IncludeSnapshotsCheckboxProps<
  T extends ValidComponent = 'checkbox',
> = PolymorphicProps<T, CheckboxRootProps<T>> & {
  show?: boolean;
};

export const IncludeSnapshotsCheckbox: Component<
  IncludeSnapshotsCheckboxProps
> = (props) => {
  const [local, others] = splitProps(props, ['show', 'class']);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn(
        'gap-2 animate-in fade-in-0 flex min-w-max items-center duration-300',
        {
          'animate-out fade-out-0 invisible': !local.show,
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
