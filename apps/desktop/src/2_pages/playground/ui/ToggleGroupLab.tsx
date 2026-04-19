import IconMdiBold from '~icons/mdi/format-bold';
import IconMdiItalic from '~icons/mdi/format-italic';
import IconMdiUnderline from '~icons/mdi/format-underline';

import { ToggleGroup, ToggleGroupItem } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const ToggleGroupLab = () => {
  return (
    <ComponentShelf title='Toggle Group'>
      <ToggleGroup multiple>
        <ToggleGroupItem value='bold' aria-label='Bold'>
          <IconMdiBold class='size-6' />
        </ToggleGroupItem>
        <ToggleGroupItem value='italic' aria-label='Italic'>
          <IconMdiItalic class='size-6' />
        </ToggleGroupItem>
        <ToggleGroupItem value='underline' aria-label='Underline'>
          <IconMdiUnderline class='size-6' />
        </ToggleGroupItem>
      </ToggleGroup>
    </ComponentShelf>
  );
};
