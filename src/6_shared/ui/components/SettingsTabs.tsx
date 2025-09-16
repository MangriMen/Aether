import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import { cn } from '@/shared/lib';

import type { TabsContentProps, TabsListProps, TabsTriggerProps } from './Tabs';

import { TabsContent, TabsList, TabsTrigger } from './Tabs';

export const SettingsTabsList = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsListProps<T>>,
) => (
  <TabsList
    {...props}
    class={cn(
      'min-w-40 justify-start bg-[unset] p-0 pl-1 pt-1 mr-8',
      props.class,
    )}
  />
);

export const SettingsTabsTrigger = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, TabsTriggerProps<T>>,
) => (
  <TabsTrigger
    {...props}
    class={cn('w-full justify-start gap-2 h-auto', props.class)}
  />
);

export const SettingsTabsContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsContentProps<T>>,
) => (
  <TabsContent
    {...props}
    class={cn(
      'flex-1 overflow-y-auto px-1 pt-1 duration-300 animate-in slide-in-from-bottom-6 data-[orientation=vertical]:mx-auto',
      props.class,
    )}
  />
);
