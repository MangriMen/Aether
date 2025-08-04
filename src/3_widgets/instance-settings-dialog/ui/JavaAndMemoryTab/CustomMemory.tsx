import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { OverridableField, type LabeledField } from '@/shared/ui';
import { useTranslation } from '@/shared/model';
import { MemoryInput } from './MemoryInput';

export type CustomMemoryProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  minValue: number;
  maxValue: number;
  value?: number | null;
  defaultValue?: number;
  onChange?: (value: number | null) => void;
};

export const CustomMemory: Component<CustomMemoryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'minValue',
    'maxValue',
    'value',
    'defaultValue',
    'onChange',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const minMemory = createMemo(() => Math.max(Math.floor(local.minValue), 0));
  const maxMemory = createMemo(() => Math.floor(local.maxValue));
  const warningMemory = createMemo(() => Math.floor(maxMemory() / 2));

  const defaultMemory = createMemo(() => local.defaultValue ?? minMemory());

  const getClampedMemory = (value: number | null) => {
    if (value === null) return null;
    return Math.max(minMemory(), Math.min(value, maxMemory()));
  };

  const handleChangeMemory = (value: number[]) => {
    local.onChange?.(getClampedMemory(value[0]));
  };

  return (
    <OverridableField<typeof LabeledField, number>
      overrideValue={null}
      value={local.value ?? null}
      defaultValue={defaultMemory()}
      onChange={local.onChange}
      class={local.class}
      fieldLabel={t('instanceSettings.memoryAllocation')}
      checkboxLabel={t('instanceSettings.customMemorySettings')}
      {...others}
    >
      {(disabled) => (
        <MemoryInput
          minValue={minMemory()}
          maxValue={maxMemory()}
          warningValue={warningMemory()}
          disabled={disabled}
          value={local.value !== null ? [local.value] : [defaultMemory()]}
          onChange={handleChangeMemory}
        />
      )}
    </OverridableField>
  );
};
