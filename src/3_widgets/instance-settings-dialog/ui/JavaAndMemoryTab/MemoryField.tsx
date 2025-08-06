import type { Component, ComponentProps } from 'solid-js';
import { createMemo, splitProps } from 'solid-js';

import { OverridableField } from '@/shared/ui';
import { useTranslation } from '@/shared/model';
import { MemoryInput } from './MemoryInput';
import { MIN_JRE_MEMORY } from '../../model';
import { bytesToMegabytes } from '../../lib';
import { useMaxRam } from '@/entities/settings';

export type MemoryFieldProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  value?: number | null;
  defaultValue?: number;
  onChange?: (value: number | null) => void;
};

export const MemoryField: Component<MemoryFieldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'value',
    'defaultValue',
    'onChange',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const maxRam = useMaxRam();

  const minMemory = createMemo(() => Math.max(Math.floor(MIN_JRE_MEMORY), 0));
  const maxMemory = createMemo(() =>
    maxRam.data ? Math.floor(bytesToMegabytes(maxRam.data)) : MIN_JRE_MEMORY,
  );
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
