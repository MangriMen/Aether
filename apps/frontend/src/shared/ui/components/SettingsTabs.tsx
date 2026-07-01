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
    class={cn(
      `
        min-w-40 p-0
        sm:static
        justify-start bg-[unset]
      `,
      props.class,
    )}
  />
);

export const SettingsTabsTrigger = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, TabsTriggerProps<T>>,
) => (
  <TabsTrigger
    {...props}
    class={cn('gap-2 w-full justify-start', props.class)}
  />
);

export const SettingsTabsContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, TabsContentProps<T>>,
) => (
  <TabsContent
    {...props}
    class={cn(
      `
        pl-6 ease-custom-expo animate-in fade-in slide-in-from-bottom-12 flex-1
        overflow-y-auto duration-200
        data-[orientation=vertical]:mx-auto
      `,
      props.class,
    )}
  />
);
