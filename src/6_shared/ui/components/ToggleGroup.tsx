import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { JSX, ValidComponent } from 'solid-js';

import * as ToggleGroupPrimitive from '@kobalte/core/toggle-group';
import { createContext, splitProps, useContext } from 'solid-js';

import { cn } from '@/shared/lib';

import { toggleVariants } from './Toggle';

const ToggleGroupContext = createContext<VariantProps<typeof toggleVariants>>({
  size: 'default',
  variant: 'default',
});

type ToggleGroupRootProps<T extends ValidComponent = 'div'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & ToggleGroupPrimitive.ToggleGroupRootProps<T> &
  VariantProps<typeof toggleVariants>;

const ToggleGroup = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ToggleGroupRootProps<T>>,
) => {
  const [local, others] = splitProps(props as ToggleGroupRootProps, [
    'class',
    'children',
    'size',
    'variant',
  ]);

  return (
    <ToggleGroupPrimitive.Root
      class={cn('flex items-center justify-center gap-1', local.class)}
      {...others}
    >
      <ToggleGroupContext.Provider
        value={{
          get size() {
            return local.size;
          },
          get variant() {
            return local.variant;
          },
        }}
      >
        {local.children}
      </ToggleGroupContext.Provider>
    </ToggleGroupPrimitive.Root>
  );
};

type ToggleGroupItemProps<T extends ValidComponent = 'button'> = {
  class?: string | undefined;
} & ToggleGroupPrimitive.ToggleGroupItemProps<T> &
  VariantProps<typeof toggleVariants>;

const ToggleGroupItem = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, ToggleGroupItemProps<T>>,
) => {
  const [local, others] = splitProps(props as ToggleGroupItemProps, [
    'class',
    'size',
    'variant',
  ]);
  const context = useContext(ToggleGroupContext);
  return (
    <ToggleGroupPrimitive.Item
      class={cn(
        toggleVariants({
          size: context.size || local.size,
          variant: context.variant || local.variant,
        }),
        'hover:bg-muted hover:text-muted-foreground data-[pressed]:bg-accent data-[pressed]:text-accent-foreground',
        local.class,
      )}
      {...others}
    />
  );
};

export type { ToggleGroupItemProps, ToggleGroupRootProps };

export { ToggleGroup, ToggleGroupItem };
