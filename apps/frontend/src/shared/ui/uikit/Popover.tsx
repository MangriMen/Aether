import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { Component, ValidComponent } from 'solid-js';

import * as PopoverPrimitive from '@kobalte/core/popover';
import { splitProps } from 'solid-js';

import { cn } from '../../lib';

const PopoverTrigger = PopoverPrimitive.Trigger;

const Popover: Component<PopoverPrimitive.PopoverRootProps> = (props) => {
  return <PopoverPrimitive.Root gutter={4} {...props} />;
};

type PopoverContentProps<T extends ValidComponent = 'div'> =
  PopoverPrimitive.PopoverContentProps<T> & { class?: string | undefined };

const PopoverContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, PopoverContentProps<T>>,
) => {
  const [local, others] = splitProps(props as PopoverContentProps, ['class']);
  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Content
        class={cn(
          `
            w-72 rounded-md bg-popover p-4 text-popover-foreground shadow-md
            data-closed:animate-out data-closed:fade-out-0
            data-closed:zoom-out-95
            data-expanded:animate-in data-expanded:fade-in-0
            data-expanded:zoom-in-95
            z-50 origin-(--kb-popover-content-transform-origin) border
            outline-none
          `,
          local.class,
        )}
        {...others}
      />
    </PopoverPrimitive.Portal>
  );
};

export { Popover, PopoverTrigger, PopoverContent };
