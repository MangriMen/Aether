import type { PolymorphicProps } from '@kobalte/core';

import { Show, splitProps, type ValidComponent } from 'solid-js';

import { cn } from '@/shared/lib';

import type { SliderRootProps } from './Slider';

import {
  Slider,
  SliderFill,
  SliderThumb,
  SliderTrack,
  SliderValueLabel,
} from './Slider';

export type CombinedSliderProps<T extends ValidComponent = 'div'> = {
  withLabel?: boolean;
} & SliderRootProps<T>;

export const CombinedSlider = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, CombinedSliderProps<T>>,
) => {
  const [local, others] = splitProps(props, ['withLabel']);

  return (
    <Slider
      class={cn('w-[300px]', others.class)}
      {...(others as PolymorphicProps<T, SliderRootProps<T>>)}
    >
      <Show when={local.withLabel}>
        <SliderValueLabel />
      </Show>
      <SliderTrack>
        <SliderFill />
        <SliderThumb />
      </SliderTrack>
    </Slider>
  );
};
