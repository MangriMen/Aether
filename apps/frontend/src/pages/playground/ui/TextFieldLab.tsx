import { TextField, TextFieldInput, TextFieldLabel } from '@/shared/ui';

import { ComponentShelf } from './ComponentShelf';

export const TextFieldLab = () => {
  return (
    <ComponentShelf title='Text Field'>
      <TextField class='max-w-sm gap-1.5 grid w-full items-center'>
        <TextFieldLabel for='email'>Email</TextFieldLabel>
        <TextFieldInput type='email' id='email' placeholder='Email' />
      </TextField>
    </ComponentShelf>
  );
};
