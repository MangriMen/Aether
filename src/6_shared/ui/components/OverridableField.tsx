import { cn } from '@/shared/lib';
import { Checkbox, LabeledField } from '@/shared/ui';
import {
  createEffect,
  createSignal,
  splitProps,
  type ComponentProps,
  type JSX,
  type ValidComponent,
} from 'solid-js';

export type OverridableFieldChildrenProps<V = unknown> = {
  get value(): V | undefined;
  get disabled(): boolean;
  get defaultValue(): V | undefined;
  onChange: (value: V) => void;
};

export type OverridableFieldProps<
  T extends ValidComponent = 'div',
  V = unknown,
  NV = unknown,
> = Omit<ComponentProps<T>, 'onChange' | 'children'> & {
  checkboxLabel?: string;
  fieldLabel?: string;
  nullValue?: V | NV;
  value?: V;
  defaultValue?: V;
  class?: string;
  onChange?: (value: V | NV) => void;
  children: (props: OverridableFieldChildrenProps<V>) => JSX.Element;
};

export const OverridableField = <
  T extends ValidComponent = 'div',
  V = unknown,
  NV = unknown,
>(
  props: OverridableFieldProps<T, V, NV>,
) => {
  const [local, others] = splitProps(props, [
    'fieldLabel',
    'checkboxLabel',
    'class',
    'nullValue',
    'value',
    'defaultValue',
    'onChange',
    'children',
  ]);

  const [isCustom, setIsCustom] = createSignal(false);

  createEffect(() => {
    setIsCustom(!!local.value);
  });

  const currentValue = () => local.value ?? local.defaultValue;

  const handleChangeIsCustom = (isChecked: boolean) => {
    setIsCustom(isChecked);
    local.onChange?.(isChecked ? currentValue() : local.nullValue);
  };

  const childrenProps: OverridableFieldChildrenProps<V> = {
    get disabled() {
      return !isCustom();
    },
    get value() {
      return currentValue();
    },
    get defaultValue() {
      return local.defaultValue;
    },
    onChange: (value: V) => {
      if (isCustom()) {
        local.onChange?.(value);
      }
    },
  };

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label={local.fieldLabel}
      {...others}
    >
      <Checkbox
        label={local.checkboxLabel}
        checked={isCustom()}
        onChange={handleChangeIsCustom}
      />
      {local.children(childrenProps)}
    </LabeledField>
  );
};
