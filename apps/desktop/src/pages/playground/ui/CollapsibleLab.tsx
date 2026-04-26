import IconMdiChevronDown from '~icons/mdi/chevron-down';

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const CollapsibleLab = () => {
  return (
    <ComponentShelf title='Collapsible'>
      <Collapsible class='w-[300px]'>
        <CollapsibleTrigger class='flex w-full items-center justify-between rounded-md border p-3'>
          <span>What is Kobalte ?</span>
          <IconMdiChevronDown />
        </CollapsibleTrigger>
        <CollapsibleContent class='mt-2 rounded-md border p-3'>
          Kobalte is a UI toolkit for building accessible web apps and design
          systems with SolidJS. It provides a set of low-level UI components and
          primitives which can be the foundation for your design system
          implementation.
        </CollapsibleContent>
      </Collapsible>
    </ComponentShelf>
  );
};
