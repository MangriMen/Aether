import type { JSX } from 'solid-js';

import { createSignal, splitProps, type Component } from 'solid-js';

import { cn } from '../../lib';
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
        'bg-card/card rounded-md hover:bg-card/hover border':
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
        <div class='gap-1 flex items-center'>
          {local.children}

          <CollapsibleIcon open={isOpened()} />
        </div>
      </CollapsibleTrigger>
      <CollapsibleContent class='gap-4 pb-4 flex flex-col'>
        {local.collapsibleContent}
      </CollapsibleContent>
    </Collapsible>
  );
};
