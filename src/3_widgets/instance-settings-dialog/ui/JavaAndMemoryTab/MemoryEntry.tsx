import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { OverridableField } from '@/shared/ui';
import { useTranslation } from '@/shared/model';
import { MemoryInput } from './MemoryInput';

export type MemoryEntryProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  minValue: number;
  maxValue: number;
  value?: number | null;
  defaultValue?: number;
  onChange?: (value: number | null) => void;
};

export const MemoryEntry: Component<MemoryEntryProps> = (props) => {
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

  const value = createMemo(() =>
    local.value != null ? [local.value] : undefined,
  );

  const defaultMemory = createMemo(() => {
    if (local.defaultValue === undefined) {
      return [minMemory()];
    }

    return [local.defaultValue];
  });

  const getClampedMemory = (value: number | null) => {
    if (value === null) return null;
    return Math.max(minMemory(), Math.min(value, maxMemory()));
  };

  const handleChangeMemory = (value: number[] | null) => {
    local.onChange?.(value ? getClampedMemory(value[0]) : value);
  };

  return (
    <OverridableField
      fieldLabel={t('instanceSettings.memoryAllocation')}
      checkboxLabel={t('instanceSettings.customMemorySettings')}
      nullValue={null}
      value={value()}
      defaultValue={defaultMemory()}
      onChange={handleChangeMemory}
      class={local.class}
      {...others}
    >
      {(props) => (
        <MemoryInput
          {...props}
          minValue={minMemory()}
          maxValue={maxMemory()}
          warningValue={warningMemory()}
        />
      )}
    </OverridableField>
  );
};
