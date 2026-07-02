import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { ValidComponent } from 'solid-js';

import * as TabsPrimitive from '@kobalte/core/tabs';
import { splitProps } from 'solid-js';

import { cn } from '../../lib';

type TabsProps<T extends ValidComponent = 'div'> =
  TabsPrimitive.TabsRootProps<T> & {
    class?: string | undefined;
  };

const Tabs = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsProps, ['class']);
  return (
    <TabsPrimitive.Root
      class={cn('data-[orientation=vertical]:flex', local.class)}
      {...others}
    />
  );
};

type TabsListProps<T extends ValidComponent = 'div'> =
  TabsPrimitive.TabsListProps<T> & {
    class?: string | undefined;
  };

const TabsList = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsListProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsListProps, ['class']);
  return (
    <TabsPrimitive.List
      class={cn(
        `
          bg-card/card text-muted-foreground h-9 gap-0.5 rounded-md p-1
          inline-flex items-center justify-center
          data-[orientation=vertical]:h-full
          data-[orientation=vertical]:flex-col
        `,
        local.class,
      )}
      {...others}
    />
  );
};

type TabsTriggerProps<T extends ValidComponent = 'button'> =
  TabsPrimitive.TabsTriggerProps<T> & {
    class?: string | undefined;
  };

const TabsTrigger = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, TabsTriggerProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsTriggerProps, ['class']);
  return (
    <TabsPrimitive.Trigger
      class={cn(
        `
          ring-offset-background
          focus-visible:ring-ring
          enabled:hover:bg-primary/solid-hover
          enabled:hover:text-primary-foreground
          data-selected:bg-primary data-selected:text-primary-foreground
          rounded-sm px-3 py-1.5 text-sm font-medium
          data-selected:shadow-sm
          inline-flex items-center justify-center whitespace-nowrap
          transition-all
          focus-visible:ring-2 focus-visible:outline-none
          disabled:pointer-events-none disabled:opacity-50
        `,
        local.class,
      )}
      {...others}
    />
  );
};

type TabsContentProps<T extends ValidComponent = 'div'> =
  TabsPrimitive.TabsContentProps<T> & {
    class?: string | undefined;
  };

const TabsContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsContentProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsContentProps, ['class']);
  return (
    <TabsPrimitive.Content
      class={cn(
        `
          ring-offset-background
          focus-visible:ring-ring
          rounded-sm
          data-[orientation=horizontal]:mt-2
          data-[orientation=vertical]:ml-2
          focus-visible:ring-2 focus-visible:ring-offset-0
          focus-visible:outline-none
          data-[orientation=vertical]:grow
        `,
        local.class,
      )}
      {...others}
    />
  );
};

type TabsIndicatorProps<T extends ValidComponent = 'div'> =
  TabsPrimitive.TabsIndicatorProps<T> & {
    class?: string | undefined;
  };

const TabsIndicator = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsIndicatorProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsIndicatorProps, ['class']);
  return (
    <TabsPrimitive.Indicator
      class={cn(
        `
          data-[orientation=horizontal]:h-0.5
          data-[orientation=vertical]:w-0.5
          absolute transition-all duration-250
          data-[orientation=horizontal]:-bottom-px
          data-[orientation=vertical]:-right-px
        `,
        local.class,
      )}
      {...others}
    />
  );
};

export type { TabsProps, TabsListProps, TabsTriggerProps, TabsContentProps };

export { Tabs, TabsList, TabsTrigger, TabsContent, TabsIndicator };
