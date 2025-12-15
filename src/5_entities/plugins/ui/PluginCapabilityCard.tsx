import type { Component, ComponentProps } from 'solid-js';

import IconMdiChevronDown from '~icons/mdi/chevron-down';
import { createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  Image,
} from '@/shared/ui';

import type { ImporterCapability, UpdaterCapability } from '../model';

export type PluginImporterProps = ComponentProps<'div'> & {
  capability: ImporterCapability | UpdaterCapability;
};

export const PluginCapabilityCard: Component<PluginImporterProps> = (props) => {
  const [local, others] = splitProps(props, ['capability', 'class']);

  const [isOpened, setIsOpened] = createSignal(false);

  return (
    <Collapsible
      class={cn('flex flex-col rounded-lg bg-secondary-dark p-2', local.class)}
      open={isOpened()}
      onOpenChange={setIsOpened}
      {...others}
    >
      <CollapsibleTrigger class='flex items-center gap-2'>
        <Image class='size-8 border' src={local.capability.icon} />
        <span class='inline-flex items-center gap-1'>
          <span>{local.capability.name}</span>
          <span class='text-muted-foreground'>({local.capability.id})</span>
        </span>
        <IconMdiChevronDown
          class={cn('ml-auto -rotate-180 transition-transform', {
            'rotate-0': isOpened(),
          })}
        />
      </CollapsibleTrigger>
      <CollapsibleContent>
        <span class='ml-0.5 text-sm text-muted-foreground'>
          {local.capability.description}
        </span>
      </CollapsibleContent>
    </Collapsible>
  );
};
