import type { Component, ComponentProps } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import { MIN_JRE_MEMORY } from '@/features/instance-settings/java-and-memory';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { useMaxRam } from '../model';
import { bytesToMegabytes } from '../model';
import { MemoryInput } from './MemoryInput';
import { OverrideCheckbox } from './OverrideCheckbox';

export type OverridableMemoryFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange'
> & {
  value?: number | null;
  defaultValue?: number;
  overridable?: boolean;
  isOverridden?: boolean;
  onChange?: (value: number | null) => void;
  onOverrideChange?: (value: boolean) => void;
};

export const OverridableMemoryField: Component<OverridableMemoryFieldProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'value',
    'defaultValue',
    'overridable',
    'isOverridden',
    'onChange',
    'onOverrideChange',
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
    <div class={cn('flex flex-col gap-1', local.class)} {...others}>
      <span class='text-lg font-medium'>
        {t('instanceSettings.memoryAllocation')}
      </span>
      <Show when={local.overridable}>
        <OverrideCheckbox
          label={t('instanceSettings.customMemorySettings')}
          checked={local.isOverridden}
          enabledValue={() => true}
          disabledValue={() => false}
          onOverrideChange={local.onOverrideChange}
        />
      </Show>
      <MemoryInput
        disabled={!local.isOverridden}
        value={value()}
        defaultValue={defaultMemory()}
        minValue={minMemory()}
        maxValue={maxMemory()}
        warningValue={warningMemory()}
        onChange={handleChangeMemory}
      />
    </div>
  );
};
