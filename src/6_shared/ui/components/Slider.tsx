import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { JSX, ValidComponent } from 'solid-js';

import * as SliderPrimitive from '@kobalte/core/slider';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Label } from '@/shared/ui';

type SliderRootProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & SliderPrimitive.SliderRootProps<T>;

const Slider = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SliderRootProps<T>>,
) => {
  const [local, others] = splitProps(props as SliderRootProps, ['class']);
  return (
    <SliderPrimitive.Root
      class={cn(
        'relative flex w-full touch-none select-none flex-col items-center',
        local.class,
      )}
      {...others}
    />
  );
};

type SliderTrackProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & SliderPrimitive.SliderTrackProps<T>;

const SliderTrack = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SliderTrackProps<T>>,
) => {
  const [local, others] = splitProps(props as SliderTrackProps, ['class']);
  return (
    <SliderPrimitive.Track
      class={cn(
        'relative h-1 w-full grow rounded-full bg-secondary data-[disabled]:opacity-50 data-[disabled]:pointer-events-none',
        local.class,
      )}
      {...others}
    />
  );
};

type SliderFillProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & SliderPrimitive.SliderFillProps<T>;

const SliderFill = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SliderFillProps<T>>,
) => {
  const [local, others] = splitProps(props as SliderFillProps, ['class']);
  return (
    <SliderPrimitive.Fill
      class={cn('absolute h-full rounded-full bg-primary', local.class)}
      {...others}
    />
  );
};

type SliderThumbProps<T extends ValidComponent = 'span'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & SliderPrimitive.SliderThumbProps<T>;

const SliderThumb = <T extends ValidComponent = 'span'>(
  props: PolymorphicProps<T, SliderThumbProps<T>>,
) => {
  const [local, others] = splitProps(props as SliderThumbProps, [
    'class',
    'children',
  ]);
  return (
    <SliderPrimitive.Thumb
      class={cn(
        'data-[disabled]:border-muted-foreground top-[-6px] block w-2 h-4 rounded-full border-2 border-primary bg-background ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:pointer-events-none disabled:opacity-50',
        local.class,
      )}
      {...others}
    >
      <SliderPrimitive.Input />
    </SliderPrimitive.Thumb>
  );
};

const SliderLabel = <T extends ValidComponent = 'label'>(
  props: PolymorphicProps<T, SliderPrimitive.SliderLabelProps<T>>,
) => {
  return <SliderPrimitive.Label as={Label} {...props} />;
};

const SliderValueLabel = <T extends ValidComponent = 'label'>(
  props: PolymorphicProps<T, SliderPrimitive.SliderValueLabelProps<T>>,
) => {
  return <SliderPrimitive.ValueLabel as={Label} {...props} />;
};

export type { SliderRootProps };

export {
  Slider,
  SliderFill,
  SliderLabel,
  SliderThumb,
  SliderTrack,
  SliderValueLabel,
};
