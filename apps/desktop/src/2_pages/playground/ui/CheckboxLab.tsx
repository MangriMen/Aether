import { Checkbox, Label } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const CheckboxLab = () => {
  return (
    <ComponentShelf title='Select'>
      <div class='flex items-start space-x-2'>
        <Checkbox id='terms1' />
        <div class='grid gap-1.5 leading-none'>
          <Label for='terms1-input'>Accept terms and conditions</Label>
          <p class='text-sm text-muted-foreground'>
            You agree to our Terms of Service and Privacy Policy.
          </p>
        </div>
      </div>
    </ComponentShelf>
  );
};
