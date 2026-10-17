import {
  Button,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/shared/ui-kit';

import { ComponentShelf } from './ComponentShelf';

export const PopoverLab = () => {
  return (
    <ComponentShelf title='Popover'>
      <Popover>
        <PopoverTrigger as={Button<'button'>}>Trigger me</PopoverTrigger>
        <PopoverContent>
          A UI toolkit for building accessible web apps and design systems with
          SolidJS.
        </PopoverContent>
      </Popover>
    </ComponentShelf>
  );
};
