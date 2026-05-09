import type { JSX } from 'solid-js';

import { createSignal, splitProps, type Component } from 'solid-js';

import { cn } from '@/shared/lib';

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '../uikit';
import { CollapsibleIcon } from './CollapsibleIcon';
import { SettingsEntry, type SettingsEntryProps } from './SettingsEntry';

export type CollapsibleSettingsEntryProps = SettingsEntryProps & {
  collapsibleContent?: JSX.Element;
};

export const CollapsibleSettingsEntry: Component<
  CollapsibleSettingsEntryProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'collapsibleContent',
    'variant',
    'class',
    'children',
  ]);

  const [isOpened, setIsOpened] = createSignal(false);

  return (
    <Collapsible
      class={cn('flex flex-col', local.class, {
        'bg-card/card border rounded-md hover:bg-card/hover':
          local.variant === 'card',
      })}
      open={isOpened()}
      onOpenChange={setIsOpened}
      {...others}
    >
      <CollapsibleTrigger
        class='p-4'
        as={SettingsEntry}
        tabIndex={-1}
        {...others}
      >
        <div class='flex items-center gap-1'>
          {local.children}

          <CollapsibleIcon open={isOpened()} />
        </div>
      </CollapsibleTrigger>
      <CollapsibleContent class='flex flex-col gap-4 pb-4'>
        {local.collapsibleContent}
      </CollapsibleContent>
    </Collapsible>
  );
};
