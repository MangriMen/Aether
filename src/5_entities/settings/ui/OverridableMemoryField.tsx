import type { Component, ComponentProps } from 'solid-js';
import { createMemo, Show, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { MemoryInput } from './MemoryInput';
import { MIN_JRE_MEMORY } from '../../../3_widgets/instance-settings-dialog/model';
import { bytesToMegabytes } from '../../../3_widgets/instance-settings-dialog/lib';
import { useMaxRam } from '@/entities/settings';
import { cn } from '@/shared/lib';
import { OverrideCheckbox } from '@/entities/settings/ui/OverrideCheckbox';

export type OverridableMemoryFieldProps = Omit<
  ComponentProps<'div'>,
  'onChange'
> & {
  value?: number | null;
  defaultValue?: number;
  onChange?: (value: number | null) => void;
  overridable?: boolean;
};

export const OverridableMemoryField: Component<OverridableMemoryFieldProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'value',
    'defaultValue',
    'onChange',
    'overridable',
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

  const isOverride = createMemo(() => local.value !== null);

  return (
    <div class={cn('flex flex-col gap-1', local.class)} {...others}>
      <span class='text-lg font-medium'>
        {t('instanceSettings.memoryAllocation')}
      </span>
      <Show when={local.overridable}>
        <OverrideCheckbox
          label={t('instanceSettings.customMemorySettings')}
          checked={isOverride()}
          enabledValue={defaultMemory}
          disabledValue={() => null}
          onOverrideChange={handleChangeMemory}
        />
      </Show>
      <MemoryInput
        disabled={!isOverride()}
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
