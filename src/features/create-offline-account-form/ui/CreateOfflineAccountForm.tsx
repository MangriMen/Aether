import { Component, ComponentProps, createSignal, splitProps } from 'solid-js';

import {
  Button,
  DialogFooter,
  TextField,
  TextFieldInput,
  TextFieldLabel,
} from '@/shared/ui';

export type CreateOfflineAccountFormProps = ComponentProps<'div'> & {
  onCreate: (username: string) => void;
};

export const CreateOfflineAccountForm: Component<
  CreateOfflineAccountFormProps
> = (props) => {
  const [local, others] = splitProps(props, ['onCreate']);
  const [username, setUsername] = createSignal<string>('');

  const handleSubmit = () => {
    local.onCreate(username());
  };

  return (
    <div class='flex flex-col gap-4' {...others}>
      <TextField class='flex flex-col gap-2'>
        <TextFieldLabel for='username'>Username</TextFieldLabel>
        <TextFieldInput
          value={username()}
          onInput={(e: InputEvent) =>
            setUsername((e.target as HTMLInputElement).value ?? '')
          }
          type='text'
          id='username'
          required
          autocomplete='off'
        />
      </TextField>

      <DialogFooter>
        <Button variant='success' onClick={handleSubmit}>
          Create
        </Button>
      </DialogFooter>
    </div>
  );
};
