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
      class={cn('data-[orientation=vertical]:flex ', local.class)}
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
        'inline-flex h-9 items-center justify-center gap-0.5 rounded-md bg-card/card p-1 text-muted-foreground data-[orientation=vertical]:h-full data-[orientation=vertical]:flex-col',
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
        'inline-flex items-center justify-center rounded-sm px-3 py-1.5 text-sm font-medium whitespace-nowrap ring-offset-background transition-all focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none enabled:hover:bg-primary/solid-hover enabled:hover:text-primary-foreground disabled:pointer-events-none disabled:opacity-50 data-selected:bg-primary data-selected:text-primary-foreground data-selected:shadow-sm',
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
        'rounded-sm ring-offset-background focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 focus-visible:outline-none data-[orientation=horizontal]:mt-2 data-[orientation=vertical]:ml-2 data-[orientation=vertical]:grow',
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
        'absolute transition-all duration-250 data-[orientation=horizontal]:-bottom-px data-[orientation=horizontal]:h-0.5 data-[orientation=vertical]:-right-px data-[orientation=vertical]:w-0.5',
        local.class,
      )}
      {...others}
    />
  );
};

export type { TabsProps, TabsListProps, TabsTriggerProps, TabsContentProps };

export { Tabs, TabsList, TabsTrigger, TabsContent, TabsIndicator };
