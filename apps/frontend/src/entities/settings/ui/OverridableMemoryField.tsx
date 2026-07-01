import type { Component, ComponentProps } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import { MIN_JRE_MEMORY } from '@/features/instance-settings/java-and-memory';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { useMaxRam } from '../model';
import { bytesToMegabytes } from '../model';
import { InheritanceLabel } from './InheritanceLabel';
import { MemoryInput } from './MemoryInput';
import { OverrideCheckbox } from './OverrideCheckbox';

export type OverridableMemoryFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange'
> & {
  value?: number;
  defaultValue?: number;
  overridable?: boolean;
  isOverridden?: boolean;
  onChange?: (value: number) => void;
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
    maxRam.data
      ? Math.floor(bytesToMegabytes(Number.parseInt(maxRam.data.totalMemory)))
      : MIN_JRE_MEMORY,
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

  const getClampedMemory = (value: number) => {
    return Math.max(minMemory(), Math.min(value, maxMemory()));
  };

  const handleChangeMemory = (value: number[]) => {
    const firstValue = value[0];

    if (!firstValue) {
      return;
    }

    local.onChange?.(getClampedMemory(firstValue));
  };

  const isInheritance = createMemo(
    () => local.overridable && !local.isOverridden,
  );

  return (
    <div class={cn('gap-1 flex flex-col', local.class)} {...others}>
      <InheritanceLabel
        label={t('instanceSettings.memoryAllocation')}
        inheritanceLabel={t('settings.usedFromDefaultSettings')}
        isInheritance={isInheritance()}
      />

      <Show when={local.overridable && local.isOverridden !== undefined}>
        <OverrideCheckbox
          label={t('instanceSettings.customMemorySettings')}
          checked={local.isOverridden}
          onOverrideChange={local.onOverrideChange}
        />
      </Show>

      <MemoryInput
        disabled={isInheritance()}
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
