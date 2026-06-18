import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { ValidComponent } from 'solid-js';

import * as SkeletonPrimitive from '@kobalte/core/skeleton';
import { splitProps } from 'solid-js';

import { cn } from '../../lib';

type SkeletonRootProps<T extends ValidComponent = 'div'> =
  SkeletonPrimitive.SkeletonRootProps<T> & { class?: string | undefined };

const Skeleton = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, SkeletonRootProps<T>>,
) => {
  const [local, others] = splitProps(props as SkeletonRootProps, ['class']);
  return (
    <SkeletonPrimitive.Root
      class={cn(
        "bg-secondary/10 data-[animate='true']:animate-pulse",
        local.class,
      )}
      {...others}
    />
  );
};

export type { SkeletonRootProps };
export { Skeleton };
