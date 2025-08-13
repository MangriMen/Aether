import type { MemorySliderProps } from '@/entities/instances';
import { MemorySlider } from '@/entities/instances';
import type { Component, ComponentProps, ValidComponent } from 'solid-js';

import {
  createEffect,
  createMemo,
  createSignal,
  mergeProps,
  splitProps,
} from 'solid-js';

import { CombinedTextField } from '@/shared/ui';

import { cn, isNil } from '@/shared/lib';

type MemoryInputBaseProps<T extends ValidComponent = 'div'> = Omit<
  ComponentProps<T>,
  'onChange'
>;
type MemorySliderInheritedProps<T extends ValidComponent = 'div'> = Pick<
  MemorySliderProps<T>,
  | 'minValue'
  | 'maxValue'
  | 'value'
  | 'defaultValue'
  | 'onChange'
  | 'warningValue'
  | 'disabled'
>;

export type MemoryInputProps<T extends ValidComponent = 'div'> =
  MemoryInputBaseProps<T> & MemorySliderInheritedProps<T>;

const DEFAULT_PROPS = {
  minValue: 0,
  maxValue: 100,
} as const;

export const MemoryInput: Component<MemoryInputProps> = (props) => {
  const [local, memorySliderProps, others] = splitProps(
    props,
    ['minValue', 'maxValue', 'defaultValue', 'value', 'disabled', 'class'],
    ['warningValue', 'onChange'],
  );
  const merged = mergeProps(DEFAULT_PROPS, local);

  const minValue = createMemo(() =>
    Math.max(merged.minValue, DEFAULT_PROPS.minValue),
  );
  const maxValue = createMemo(() => merged.maxValue);
  const defaultValue = createMemo(() => merged.defaultValue?.[0] ?? minValue());

  const sliderDefaultValue = createMemo(() => [defaultValue()]);

  const sliderValue = createMemo(() =>
    isNil(merged.value) ? sliderDefaultValue() : merged.value,
  );

  const [textFieldValue, setTextFieldValue] = createSignal('0');

  const textFieldDefaultValue = createMemo(() => String(defaultValue()));

  createEffect(() => {
    const value = isNil(merged.value)
      ? textFieldDefaultValue()
      : String(merged.value);

    setTextFieldValue(value);
  });

  const handleChangeTextField = (e: Event) => {
    if (
      e.type === 'keypress' &&
      e instanceof KeyboardEvent &&
      e.key !== 'Enter'
    ) {
      return;
    }

    const input = e.target as HTMLInputElement;
    const value = Number.parseInt(input.value);
    if (Number.isNaN(value)) {
      return;
    }

    memorySliderProps.onChange?.([value]);
  };

  return (
    <div class={cn('flex items-start gap-4', merged.class)} {...others}>
      <MemorySlider
        class='mt-4'
        disabled={merged.disabled}
        minValue={minValue()}
        maxValue={maxValue()}
        value={sliderValue()}
        {...memorySliderProps}
      />
      <CombinedTextField
        class='w-[9ch]'
        disabled={merged.disabled}
        value={textFieldValue()}
        onChange={setTextFieldValue}
        inputProps={{
          type: 'text',
          onBlur: handleChangeTextField,
          onKeyPress: handleChangeTextField,
        }}
      />
    </div>
  );
};
