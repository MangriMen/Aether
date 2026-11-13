import type { PolymorphicProps } from '@kobalte/core';
import type { SliderRootProps } from '@kobalte/core/slider';
import type { Component, ValidComponent } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  Slider,
  SliderFill,
  SliderLabel,
  SliderThumb,
  SliderTrack,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

export type MemorySliderProps<T extends ValidComponent = 'div'> =
  PolymorphicProps<T, SliderRootProps<T>> & {
    warningValue?: number;
  };

export const MemorySlider: Component<MemorySliderProps> = (props) => {
  const [local, others] = splitProps(props, ['warningValue', 'class']);

  const [{ t }] = useTranslation();

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
      <SliderLabel class='sr-only'>
        {t('instanceSettings.memoryAllocation')}
      </SliderLabel>
      <SliderTrack>
        <SliderFill
          class={cn('transition-[background-color]', {
            'bg-warning': isWarningLimitPassed(),
          })}
        />
        <SliderThumb
          aria-valuetext={
            t('instanceSettings.memoryMb', { value: value()[0] ?? 0 }) ?? ''
          }
          class={cn({
            'border-warning': isWarningLimitPassed(),
          })}
        />
      </SliderTrack>
      <div aria-hidden='true' class='flex w-full justify-between'>
        <span>
          {t('instanceSettings.memoryMb', { value: props.minValue ?? 0 })}
        </span>
        <span>
          {t('instanceSettings.memoryMb', { value: props.maxValue ?? 0 })}
        </span>
      </div>
    </Slider>
  );
};
