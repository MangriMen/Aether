import type { Component, ComponentProps, JSX } from 'solid-js';

import { Icon } from '@iconify-icon/solid';
import MdiChevronDownIcon from '@iconify/icons-mdi/chevron-down';
import { createEffect, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/shared/ui';

export type CollapsibleSettingsPaneProps = ComponentProps<'div'> & {
  label?: JSX.Element;
  defaultOpened?: boolean;
};

export const CollapsibleSettingsPane: Component<
  CollapsibleSettingsPaneProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'label',
    'defaultOpened',
    'class',
    'children',
  ]);

  const [isOpened, setIsOpened] = createSignal(true);

  createEffect(() => {
    setIsOpened(!!local.defaultOpened);
  });

  return (
    <Collapsible
      open={isOpened()}
      onOpenChange={setIsOpened}
      class={cn(
        'flex flex-col rounded-lg bg-secondary-dark px-6 py-4 gap-2',
        local.class,
      )}
      {...others}
    >
      <CollapsibleTrigger class='justify-center text-left'>
        <div class='flex justify-between'>
          <h2 class='text-xl font-bold'>{local.label}</h2>
          <Icon
            class={cn(
              'aspect-square h-full text-2xl -rotate-180 transition-transform',
              {
                'rotate-0': isOpened(),
              },
            )}
            icon={MdiChevronDownIcon}
          />
        </div>
      </CollapsibleTrigger>
      <CollapsibleContent>{local.children}</CollapsibleContent>
    </Collapsible>
  );
};
