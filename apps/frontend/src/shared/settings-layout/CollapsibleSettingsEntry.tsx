import type { JSX } from 'solid-js';

import {
  createMemo,
  createSignal,
  createUniqueId,
  splitProps,
  type Component,
} from 'solid-js';

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  CollapsibleIcon,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import {
  SettingsEntry,
  SettingsEntryTitleContext,
  type SettingsEntryProps,
} from './SettingsEntry';

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
  // The trigger renders its children outside the entry's own context, so provide it here.
  const titleId = createUniqueId();
  const labelId = createMemo(() => (props.title ? titleId : undefined));

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
        titleId={titleId}
        {...others}
      >
        <div class='gap-1 flex items-center'>
          <SettingsEntryTitleContext.Provider value={labelId}>
            {local.children}
          </SettingsEntryTitleContext.Provider>

          <CollapsibleIcon open={isOpened()} />
        </div>
      </CollapsibleTrigger>
      <CollapsibleContent class='gap-4 pb-4 flex flex-col'>
        {local.collapsibleContent}
      </CollapsibleContent>
    </Collapsible>
  );
};
