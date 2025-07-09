import type { PolymorphicProps } from '@kobalte/core';
import type { SliderRootProps } from '@kobalte/core/slider';
import type { Component, ValidComponent } from 'solid-js';
import { createEffect, createMemo, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Slider,
  SliderFill,
  SliderLabel,
  SliderThumb,
  SliderTrack,
} from '@/shared/ui';

export type MemorySliderProps<T extends ValidComponent = 'div'> =
  PolymorphicProps<T, SliderRootProps<T>> & {
    warningValue?: number;
  };

export const MemorySlider: Component<MemorySliderProps> = (props) => {
  const [local, others] = splitProps(props, [
    'warningValue',
    'onChange',
    'class',
  ]);

  const [value, setValue] = createSignal<number[]>();

  const isValueGreaterThanWarning = createMemo(() => {
    if (!local.warningValue) {
      return;
    }

    const val = value();
    if (val?.length !== 1) {
      return;
    }

    const singleValue = val[0];
    return singleValue >= local.warningValue;
  });

  const onChange = (value: number[]) => {
    setValue(value);
    local.onChange?.(value);
  };

  createEffect(() => {
    setValue(others.value ?? others.defaultValue ?? [others.minValue ?? 0]);
  });

  return (
    <Slider
      class={cn('flex flex-col gap-3', local.class)}
      onChange={onChange}
      {...others}
    >
      <SliderTrack>
        <SliderFill
          class={cn({
            'bg-warning-foreground': local.warningValue
              ? isValueGreaterThanWarning()
              : false,
          })}
        />
        <SliderThumb
          class={cn({
            'border-warning-foreground': local.warningValue
              ? isValueGreaterThanWarning()
              : false,
          })}
        />
      </SliderTrack>
      <div class='flex w-full justify-between'>
        <SliderLabel>{props.minValue} Mb</SliderLabel>
        <SliderLabel>{props.maxValue} Mb</SliderLabel>
      </div>
    </Slider>
  );
};
