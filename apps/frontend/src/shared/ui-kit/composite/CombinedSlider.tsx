import type { PolymorphicProps } from '@kobalte/core';

import { Show, splitProps, type ValidComponent } from 'solid-js';

import type { SliderRootProps } from '../primitives/Slider';

import {
  Slider,
  SliderFill,
  SliderThumb,
  SliderTrack,
  SliderValueLabel,
} from '../primitives/Slider';

export type CombinedSliderProps<T extends ValidComponent = 'div'> =
  SliderRootProps<T> & {
    withLabel?: boolean;
  };

export const CombinedSlider = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, CombinedSliderProps<T>>,
) => {
  const [local, others] = splitProps(props, ['withLabel']);

  return (
    <Slider {...(others as PolymorphicProps<T, SliderRootProps<T>>)}>
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
