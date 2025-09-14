import type { Component, ComponentProps } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import { useMaxRam } from '@/entities/settings';
import { cn, useIsCustomCheckbox } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Checkbox, LabeledField } from '@/shared/ui';

import { bytesToMegabytes } from '../../lib';
import { MIN_JRE_MEMORY } from '../../model';
import { MemoryInput } from './MemoryInput';

export type MemoryFieldProps = {
  defaultValue?: number;
  onChange?: (value: null | number) => void;
  value?: null | number;
} & Omit<ComponentProps<'div'>, 'onChange'>;

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

  const getClampedMemory = (value: null | number) => {
    if (value === null) return null;
    return Math.max(minMemory(), Math.min(value, maxMemory()));
  };

  const handleChangeMemory = (value: null | number[]) => {
    local.onChange?.(value ? getClampedMemory(value[0]) : value);
  };

  const [isCustom, setIsCustom] = useIsCustomCheckbox({
    isCustom: () => local.value !== null,
    onChange: (isCustom) =>
      handleChangeMemory(isCustom ? defaultMemory() : null),
  });

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label={t('instanceSettings.memoryAllocation')}
      {...others}
    >
      <Checkbox
        checked={isCustom()}
        label={t('instanceSettings.customMemorySettings')}
        onChange={setIsCustom}
      />
      <MemoryInput
        defaultValue={defaultMemory()}
        disabled={!isCustom()}
        maxValue={maxMemory()}
        minValue={minMemory()}
        onChange={handleChangeMemory}
        value={value()}
        warningValue={warningMemory()}
      />
    </LabeledField>
  );
};
