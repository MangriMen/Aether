import { TextField, TextFieldInput, TextFieldLabel } from '../../../shared/ui';
import { ComponentShelf } from './ComponentShelf';

export const TextFieldLab = () => {
  return (
    <ComponentShelf title='Text Field'>
      <TextField class='grid w-full max-w-sm items-center gap-1.5'>
        <TextFieldLabel for='email'>Email</TextFieldLabel>
        <TextFieldInput type='email' id='email' placeholder='Email' />
      </TextField>
    </ComponentShelf>
  );
};
