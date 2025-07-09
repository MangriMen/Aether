import type { Component, ComponentProps } from 'solid-js';
import { createEffect, createMemo, createSignal, splitProps } from 'solid-js';

import { cn, stringToNumber } from '@/shared/lib';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

import { MemorySlider } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

export type CustomMemoryProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  minValue: number;
  maxValue: number;
  value?: number | null;
  defaultValue?: number;
  onChange?: (value: number | null) => void;
};

const CustomMemory: Component<CustomMemoryProps> = (props) => {
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

  const [isCustom, setIsCustom] = createSignal(false);

  createEffect(() => {
    setIsCustom(Boolean(local.value));
  });

  const getClampedMemory = (value: number | null) => {
    if (value === null) {
      return value;
    }

    if (value < minMemory()) {
      return minMemory();
    } else if (value > maxMemory()) {
      return maxMemory();
    }

    return value;
  };

  const setMemoryValue = (value: number | null) => {
    local.onChange?.(getClampedMemory(value));
  };

  const handleChangeIsCustom = (value: boolean) => {
    setIsCustom(value);

    if (value) {
      const clampedMemory = getClampedMemory(defaultMemory() ?? 0);
      local.onChange?.(clampedMemory);
    } else {
      local.onChange?.(null);
    }
  };

  const handleChangeTextField = (e: Event) => {
    const input = e.target as HTMLInputElement;
    const value = input.value;
    if (typeof value !== 'string') {
      return;
    }
    const parsedValue = stringToNumber(value);
    setMemoryValue(parsedValue);
  };

  const sliderDefaultValue = createMemo(() => [defaultMemory()]);

  const sliderValue = createMemo(() =>
    local.value !== null && local.value !== undefined
      ? [local.value]
      : sliderDefaultValue(),
  );

  const [textFieldValue, setTextFieldValue] = createSignal('0');

  const textFieldDefaultValue = createMemo(() => String(defaultMemory()));
  createEffect(() => {
    const value =
      local.value !== null && local.value !== undefined
        ? String(local.value)
        : textFieldDefaultValue();
    setTextFieldValue(value);
  });

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label={t('instanceSettings.memoryAllocation')}
      {...others}
    >
      <Checkbox
        label={t('instanceSettings.customMemorySettings')}
        checked={isCustom()}
        onChange={handleChangeIsCustom}
      />

      <div class='flex items-start gap-4'>
        <MemorySlider
          class='mt-4'
          minValue={minMemory()}
          maxValue={maxMemory()}
          warningValue={warningMemory()}
          disabled={!isCustom()}
          value={sliderValue()}
          onChange={(value) => setMemoryValue(value[0])}
        />
        <CombinedTextField
          class='w-[9ch]'
          disabled={!isCustom()}
          value={textFieldValue()}
          onChange={setTextFieldValue}
          inputProps={{
            type: 'text',
            onBlur: (e) => handleChangeTextField(e),
            onKeyPress: (e) => e.key === 'Enter' && handleChangeTextField(e),
          }}
        />
      </div>
    </LabeledField>
  );
};

export default CustomMemory;
