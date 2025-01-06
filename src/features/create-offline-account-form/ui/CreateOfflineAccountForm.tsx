import {
  createForm,
  reset,
  SubmitHandler,
  zodForm,
} from '@modular-forms/solid';
import { Component, ComponentProps, splitProps } from 'solid-js';

import {
  Button,
  DialogFooter,
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '@/shared/ui';

import {
  CreateOfflineAccountFormSchema,
  CreateOfflineAccountFormValues,
} from '../model';

export type CreateOfflineAccountFormProps = ComponentProps<'div'> & {
  onCreate: (username: string) => void;
  onCancel: () => void;
};

export const CreateOfflineAccountForm: Component<
  CreateOfflineAccountFormProps
> = (props) => {
  const [local, others] = splitProps(props, ['onCreate', 'onCancel']);

  const [form, { Form, Field }] = createForm<CreateOfflineAccountFormValues>({
    validate: zodForm(CreateOfflineAccountFormSchema),
  });

  const handleSubmit: SubmitHandler<CreateOfflineAccountFormValues> = (
    values,
  ) => {
    local.onCreate(values.username);
  };

  const handleCancel = () => {
    reset(form);
    local.onCancel();
  };

  return (
    <Form onSubmit={handleSubmit}>
      <div class='flex flex-col gap-4' {...others}>
        <Field name='username'>
          {(field, props) => (
            <TextField
              validationState={field.error ? 'invalid' : 'valid'}
              class='flex flex-col gap-3'
            >
              <TextFieldLabel for='username'>Username</TextFieldLabel>
              <TextFieldInput
                id='username'
                autocomplete='off'
                required
                type='text'
                value={field.value}
                {...props}
              />
              <TextFieldErrorMessage>{field.error}</TextFieldErrorMessage>
            </TextField>
          )}
        </Field>

        <DialogFooter>
          <Button type='submit' variant='success'>
            Create
          </Button>
          <Button onClick={handleCancel}>Cancel</Button>
        </DialogFooter>
      </div>
    </Form>
  );
};
