import type { Component, ComponentProps } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import { useMaxRam } from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { cn } from '@/shared/utils';

import { bytesToMegabytes, MIN_JRE_MEMORY } from '../model/memory';
import { InheritanceLabel } from './InheritanceLabel';
import { MemoryInput } from './MemoryInput';
import { OverrideCheckbox } from './OverrideCheckbox';

/** Slider range shown while the amount of RAM is unknown. */
const FALLBACK_SLIDER_MAX = 32_768;

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
  // Without RAM data (loading or failed) the value is not limited from above:
  // a guessed cap would silently lower a saved allocation.
  const ramMemory = createMemo(() => {
    const total = maxRam.data
      ? Number.parseInt(maxRam.data.totalMemory)
      : Number.NaN;

    return Number.isFinite(total) && total > 0
      ? Math.floor(bytesToMegabytes(total))
      : undefined;
  });
  const sliderMaxMemory = createMemo(
    () => ramMemory() ?? Math.max(FALLBACK_SLIDER_MAX, local.value ?? 0),
  );
  const warningMemory = createMemo(() => {
    const ram = ramMemory();
    return ram === undefined ? undefined : Math.floor(ram / 2);
  });

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
    return Math.max(minMemory(), Math.min(value, ramMemory() ?? value));
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
        maxValue={sliderMaxMemory()}
        warningValue={warningMemory()}
        onChange={handleChangeMemory}
      />
    </div>
  );
};
