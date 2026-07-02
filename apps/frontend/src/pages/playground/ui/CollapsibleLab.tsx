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
      <Collapsible class='w-75'>
        <CollapsibleTrigger
          class='
          rounded-md p-3 flex w-full items-center justify-between border
        '
        >
          <span>What is Kobalte ?</span>
          <IconMdiChevronDown />
        </CollapsibleTrigger>
        <CollapsibleContent class='mt-2 rounded-md p-3 border'>
          Kobalte is a UI toolkit for building accessible web apps and design
          systems with SolidJS. It provides a set of low-level UI components and
          primitives which can be the foundation for your design system
          implementation.
        </CollapsibleContent>
      </Collapsible>
    </ComponentShelf>
  );
};
