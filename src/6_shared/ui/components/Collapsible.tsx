import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import * as CollapsiblePrimitive from '@kobalte/core/collapsible';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const Collapsible = CollapsiblePrimitive.Root;

const CollapsibleTrigger = CollapsiblePrimitive.Trigger;

type CollapsibleContentProps<T extends ValidComponent = 'div'> =
  CollapsiblePrimitive.CollapsibleContentProps<T> & {
    class?: string | undefined;
  };

const CollapsibleContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, CollapsibleContentProps<T>>,
) => {
  const [local, others] = splitProps(props as CollapsibleContentProps, [
    'class',
  ]);
  return (
    <CollapsiblePrimitive.Content
      class={cn(
        'animate-slide-up overflow-hidden data-[expanded]:animate-slide-down',
        local.class,
      )}
      {...others}
    />
  );
};
export type { CollapsibleContentProps };
export { Collapsible, CollapsibleTrigger, CollapsibleContent };
