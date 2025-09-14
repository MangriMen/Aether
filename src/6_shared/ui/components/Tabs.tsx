import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { ValidComponent } from 'solid-js';

import * as TabsPrimitive from '@kobalte/core/tabs';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

type TabsProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & TabsPrimitive.TabsRootProps<T>;

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

type TabsListProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & TabsPrimitive.TabsListProps<T>;

const TabsList = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsListProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsListProps, ['class']);
  return (
    <TabsPrimitive.List
      class={cn(
        'inline-flex data-[orientation=vertical]:flex-col data-[orientation=vertical]:h-full h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground',
        local.class,
      )}
      {...others}
    />
  );
};

type TabsTriggerProps<T extends ValidComponent = 'button'> = {
  class?: string | undefined;
} & TabsPrimitive.TabsTriggerProps<T>;

const TabsTrigger = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, TabsTriggerProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsTriggerProps, ['class']);
  return (
    <TabsPrimitive.Trigger
      class={cn(
        'inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0 disabled:pointer-events-none disabled:opacity-50 data-[selected]:bg-primary data-[selected]:text-primary-foreground data-[selected]:shadow-sm',
        local.class,
      )}
      {...others}
    />
  );
};

type TabsContentProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & TabsPrimitive.TabsContentProps<T>;

const TabsContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsContentProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsContentProps, ['class']);
  return (
    <TabsPrimitive.Content
      class={cn(
        'data-[orientation=vertical]:grow data-[orientation=vertical]:ml-2 data-[orientation=horizontal]:mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0',
        local.class,
      )}
      {...others}
    />
  );
};

type TabsIndicatorProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & TabsPrimitive.TabsIndicatorProps<T>;

const TabsIndicator = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsIndicatorProps<T>>,
) => {
  const [local, others] = splitProps(props as TabsIndicatorProps, ['class']);
  return (
    <TabsPrimitive.Indicator
      class={cn(
        'duration-250ms absolute transition-all data-[orientation=horizontal]:-bottom-px data-[orientation=vertical]:-right-px data-[orientation=horizontal]:h-[2px] data-[orientation=vertical]:w-[2px]',
        local.class,
      )}
      {...others}
    />
  );
};

export type { TabsContentProps, TabsListProps, TabsProps, TabsTriggerProps };

export { Tabs, TabsContent, TabsIndicator, TabsList, TabsTrigger };
