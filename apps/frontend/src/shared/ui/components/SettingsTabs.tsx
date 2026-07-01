import type { PolymorphicProps } from '@kobalte/core';
import type { ValidComponent } from 'solid-js';

import type {
  TabsContentProps,
  TabsListProps,
  TabsTriggerProps,
} from '../uikit/Tabs';

import { cn } from '../../lib';
import { TabsContent, TabsList, TabsTrigger } from '../uikit/Tabs';

export const SettingsTabsList = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsListProps<T>>,
) => (
  <TabsList
    {...props}
    class={cn('min-w-40 justify-start bg-[unset] p-0 sm:static ', props.class)}
  />
);

export const SettingsTabsTrigger = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, TabsTriggerProps<T>>,
) => (
  <TabsTrigger
    {...props}
    class={cn('w-full justify-start gap-2', props.class)}
  />
);

export const SettingsTabsContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsContentProps<T>>,
) => (
  <TabsContent
    {...props}
    class={cn(
      'flex-1 overflow-y-auto pl-6 duration-200 ease-custom-expo animate-in fade-in slide-in-from-bottom-12 data-[orientation=vertical]:mx-auto',
      props.class,
    )}
  />
);
