import type { PolymorphicProps } from '@kobalte/core';
import type { SliderRootProps } from '@kobalte/core/slider';
import type { Component, ValidComponent } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

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
  const [local, others] = splitProps(props, ['warningValue', 'class']);

  const value = createMemo(
    () => others.value ?? others.defaultValue ?? [others.minValue ?? 0],
  );

  const isWarningLimitPassed = createMemo(() => {
    if (local.warningValue === undefined) {
      return false;
    }

    const firstValue = value()[0];

    return firstValue !== undefined ? firstValue >= local.warningValue : false;
  });

  return (
    <Slider class={cn('flex flex-col', local.class)} {...others}>
      <SliderTrack>
        <SliderFill
          class={cn('transition-[background-color]', {
            'bg-warning': isWarningLimitPassed(),
          })}
        />
        <SliderThumb
          class={cn({
            'border-warning': isWarningLimitPassed(),
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
