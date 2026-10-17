import type { PolymorphicProps } from '@kobalte/core';

import { type Accessor, Show, splitProps, type ValidComponent } from 'solid-js';

import { createOverrideHandler } from '@/shared/lib';
import { Checkbox, type CheckboxRootProps } from '@/shared/ui';

export type OverrideCheckboxProps<
  T extends ValidComponent = 'div',
  TEnabledValue = boolean,
  TDisabledValue = boolean,
> = CheckboxRootProps<T> & {
  enabledValue?: Accessor<TEnabledValue>;
  disabledValue?: Accessor<TDisabledValue>;
  onOverrideChange?: (value: TEnabledValue | TDisabledValue) => void;
};

export const OverrideCheckbox = <
  T extends ValidComponent = 'div',
  TEnabledValue = boolean,
  TDisabledValue = boolean,
>(
  props: PolymorphicProps<
    T,
    OverrideCheckboxProps<T, TEnabledValue, TDisabledValue>
  >,
) => {
  const [local, others] = splitProps(props, [
    'enabledValue',
    'disabledValue',
    'onOverrideChange',
    'label',
    'onChange',
  ]);

  const handleOverrideChange = createOverrideHandler(
    () => local.enabledValue?.() ?? true,
    () => local.disabledValue?.() ?? false,
    (value) => local.onOverrideChange?.(value),
  );

  const handleChange = (isChecked: boolean) => {
    local.onChange?.(isChecked);
    handleOverrideChange(isChecked);
  };

  return (
    <Checkbox
      label={
        <Show when={typeof local.label === 'string'} fallback={local.label}>
          <span class='text-muted-foreground'>{local.label}</span>
        </Show>
      }
      onChange={handleChange}
      {...(others as PolymorphicProps<T, CheckboxRootProps<T>>)}
    />
  );
};
