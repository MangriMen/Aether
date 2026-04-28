import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { Component, JSX, ValidComponent } from 'solid-js';

import * as ProgressPrimitive from '@kobalte/core/progress';
import { cva } from 'class-variance-authority';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { Label } from './Label';

const progressVariants = cva('', {
  variants: {
    variant: {
      default: 'bg-primary',
      error: 'bg-error',
    },
  },
  defaultVariants: {
    variant: 'default',
  },
});

type ProgressRootProps<T extends ValidComponent = 'div'> =
  ProgressPrimitive.ProgressRootProps<T> &
    VariantProps<typeof progressVariants> & { children?: JSX.Element };

const Progress = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ProgressRootProps<T>>,
) => {
  const [local, others] = splitProps(props as ProgressRootProps, [
    'variant',
    'children',
  ]);
  return (
    <ProgressPrimitive.Root {...others}>
      {local.children}
      <ProgressPrimitive.Track class='relative h-2 w-full overflow-hidden rounded-full bg-secondary/secondary'>
        <ProgressPrimitive.Fill
          class={cn(
            progressVariants({ variant: local.variant }),
            'h-full w-[var(--kb-progress-fill-width)] flex-1 transition-all',
          )}
        />
      </ProgressPrimitive.Track>
    </ProgressPrimitive.Root>
  );
};

const ProgressLabel: Component<ProgressPrimitive.ProgressLabelProps> = (
  props,
) => {
  return <ProgressPrimitive.Label as={Label} {...props} />;
};

const ProgressValueLabel: Component<
  ProgressPrimitive.ProgressValueLabelProps
> = (props) => {
  return <ProgressPrimitive.ValueLabel as={Label} {...props} />;
};

export { Progress, ProgressLabel, ProgressValueLabel };
