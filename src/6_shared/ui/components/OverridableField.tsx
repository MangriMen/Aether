import { cn } from '@/shared/lib';
import { Checkbox, LabeledField } from '@/shared/ui';
import {
  createSignal,
  splitProps,
  type ComponentProps,
  type JSX,
  type ValidComponent,
} from 'solid-js';
export type OverridableFieldProps<
  T extends ValidComponent = 'div',
  V = unknown,
> = Omit<ComponentProps<T>, 'children'> & {
  overrideValue: V;
  value: V | null; // Текущее значение (контролируемое)
  defaultValue?: V; // Начальное значение (если value не указано)
  checkboxLabel?: string;
  fieldLabel?: string;
  class?: string;
  onChange?: (value: V | null) => void;
  children: (disabled: boolean, value: V | null) => JSX.Element;
};

export const OverridableField = <T extends ValidComponent = 'div', V = unknown>(
  props: OverridableFieldProps<T, V>,
) => {
  const [local, others] = splitProps(props, [
    'fieldLabel',
    'checkboxLabel',
    'class',
    'children',
    'overrideValue',
    'value',
    'defaultValue',
    'onChange',
  ]);

  const [isCustom, setIsCustom] = createSignal(false);

  const currentValue = () =>
    local.value ?? local.defaultValue ?? local.overrideValue;

  const handleChangeIsCustom = (isChecked: boolean) => {
    setIsCustom(isChecked);
    local.onChange?.(isChecked ? currentValue() : local.overrideValue);
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
      {local.children(!isCustom(), currentValue())}
    </LabeledField>
  );
};
