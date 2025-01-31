import type { PolymorphicProps } from '@kobalte/core';
import type { SliderRootProps } from '@kobalte/core/slider';
import type { Component, ValidComponent } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Slider,
  SliderFill,
  SliderLabel,
  SliderThumb,
  SliderTrack,
} from '@/shared/ui';

export type MemorySliderProps<T extends ValidComponent = 'div'> =
  PolymorphicProps<T, SliderRootProps<T>>;

const MemorySlider: Component<MemorySliderProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <Slider class={cn('flex flex-col gap-3', local.class)} {...others}>
      <SliderTrack>
        <SliderFill />
        <SliderThumb />
      </SliderTrack>
      <div class='flex w-full justify-between'>
        <SliderLabel>{props.minValue} Mb</SliderLabel>
        <SliderLabel>{props.maxValue} Mb</SliderLabel>
      </div>
    </Slider>
  );
};

export default MemorySlider;
