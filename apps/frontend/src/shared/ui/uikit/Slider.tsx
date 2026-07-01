import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { JSX, ValidComponent } from 'solid-js';

import * as SliderPrimitive from '@kobalte/core/slider';
import { splitProps } from 'solid-js';

import { cn } from '../../lib';
import { Label } from './Label';

type SliderRootProps<T extends ValidComponent = 'div'> =
  SliderPrimitive.SliderRootProps<T> & {
    class?: string | undefined;
  };

const Slider = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SliderRootProps<T>>,
) => {
  const [local, others] = splitProps(props as SliderRootProps, ['class']);
  return (
    <SliderPrimitive.Root
      class={cn(
        'relative flex w-full touch-none flex-col items-center select-none',
        local.class,
      )}
      {...others}
    />
  );
};

type SliderTrackProps<T extends ValidComponent = 'div'> =
  SliderPrimitive.SliderTrackProps<T> & {
    class?: string | undefined;
  };

const SliderTrack = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SliderTrackProps<T>>,
) => {
  const [local, others] = splitProps(props as SliderTrackProps, ['class']);
  return (
    <SliderPrimitive.Track
      class={cn(
        `
          my-4 h-1.5 bg-secondary/secondary relative w-full grow rounded-full
          data-disabled:pointer-events-none data-disabled:opacity-50
        `,
        local.class,
      )}
      {...others}
    >
      <div class='inset-y-0 h-9 rounded-md absolute my-auto w-full grow' />
      {props.children}
    </SliderPrimitive.Track>
  );
};

type SliderFillProps<T extends ValidComponent = 'div'> =
  SliderPrimitive.SliderFillProps<T> & {
    class?: string | undefined;
  };

const SliderFill = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SliderFillProps<T>>,
) => {
  const [local, others] = splitProps(props as SliderFillProps, ['class']);
  return (
    <SliderPrimitive.Fill
      class={cn('bg-primary absolute h-full rounded-full', local.class)}
      {...others}
    />
  );
};

type SliderThumbProps<T extends ValidComponent = 'span'> =
  SliderPrimitive.SliderThumbProps<T> & {
    class?: string | undefined;
    children?: JSX.Element;
  };

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
        `
          -top-1.5 h-5 w-3 border-primary bg-background ring-offset-background
          focus-visible:ring-ring
          data-disabled:border-primary
          block rounded-full border-[3px] transition-colors
          focus-visible:ring-2 focus-visible:ring-offset-0
          focus-visible:outline-none
          disabled:pointer-events-none disabled:opacity-50
        `,
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
  SliderTrack,
  SliderFill,
  SliderThumb,
  SliderLabel,
  SliderValueLabel,
};
