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

import type { ImporterCapability } from '../model';

export type PluginImporterProps = ComponentProps<'div'> & {
  importer: ImporterCapability;
};

export const PluginImporter: Component<PluginImporterProps> = (props) => {
  const [local, others] = splitProps(props, ['importer', 'class']);

  const [isOpened, setIsOpened] = createSignal(false);

  return (
    <Collapsible
      class={cn('flex flex-col rounded-lg bg-secondary-dark p-2', local.class)}
      open={isOpened()}
      onOpenChange={setIsOpened}
      {...others}
    >
      <CollapsibleTrigger class='flex items-center gap-2'>
        <Image class='size-8 border' src={local.importer.icon} />
        <span class='inline-flex items-center gap-1'>
          <span>{local.importer.name}</span>
          <span class='text-muted-foreground'>({local.importer.id})</span>
        </span>
        <IconMdiChevronDown
          class={cn('ml-auto -rotate-180 transition-transform', {
            'rotate-0': isOpened(),
          })}
        />
      </CollapsibleTrigger>
      <CollapsibleContent>
        <span class='ml-0.5 text-sm text-muted-foreground'>
          {local.importer.description}
        </span>
      </CollapsibleContent>
    </Collapsible>
  );
};
