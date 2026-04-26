import { Button } from '../../../shared/ui';
import { Tooltip, TooltipContent, TooltipTrigger } from '../../../shared/ui';
import { ComponentShelf } from './ComponentShelf';

export const TooltipLab = () => {
  return (
    <ComponentShelf title='Tooltip'>
      <Tooltip>
        <TooltipTrigger as={Button<'button'>} variant='outline'>
          Trigger
        </TooltipTrigger>
        <TooltipContent>
          <p>Tooltip content</p>
        </TooltipContent>
      </Tooltip>
    </ComponentShelf>
  );
};
