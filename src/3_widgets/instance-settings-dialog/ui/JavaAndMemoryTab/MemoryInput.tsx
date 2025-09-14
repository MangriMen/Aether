import type { Component, ComponentProps, ValidComponent } from 'solid-js';

import {
  createEffect,
  createMemo,
  createSignal,
  mergeProps,
  splitProps,
} from 'solid-js';

import type { MemorySliderProps } from '@/entities/instances';

import { MemorySlider } from '@/entities/instances';
import { cn, isNil } from '@/shared/lib';
import { CombinedTextField } from '@/shared/ui';

export type MemoryInputProps<T extends ValidComponent = 'div'> =
  MemoryInputBaseProps<T> & MemorySliderInheritedProps<T>;
type MemoryInputBaseProps<T extends ValidComponent = 'div'> = Omit<
  ComponentProps<T>,
  'onChange'
>;

type MemorySliderInheritedProps<T extends ValidComponent = 'div'> = Pick<
  MemorySliderProps<T>,
  | 'defaultValue'
  | 'disabled'
  | 'maxValue'
  | 'minValue'
  | 'onChange'
  | 'value'
  | 'warningValue'
>;

const DEFAULT_PROPS = {
  maxValue: 100,
  minValue: 0,
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
        maxValue={maxValue()}
        minValue={minValue()}
        value={sliderValue()}
        {...memorySliderProps}
      />
      <CombinedTextField
        class='w-[9ch]'
        disabled={merged.disabled}
        inputProps={{
          onBlur: handleChangeTextField,
          onKeyPress: handleChangeTextField,
          type: 'text',
        }}
        onChange={setTextFieldValue}
        value={textFieldValue()}
      />
    </div>
  );
};
