import type { SubmitHandler } from '@modular-forms/solid';
import type { Component, ComponentProps } from 'solid-js';

import { createForm, reset, zodForm } from '@modular-forms/solid';
import { splitProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import {
  Button,
  DialogFooter,
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '@/shared/ui';

import type {
  CreateOfflineAccountFormSchemaErrors,
  CreateOfflineAccountFormValues,
} from '../model';

import { CreateOfflineAccountFormSchema } from '../model';

export type CreateOfflineAccountFormProps = {
  onCancel: () => void;
  onCreate: (username: string) => void;
} & ComponentProps<'div'>;

export const CreateOfflineAccountForm: Component<
  CreateOfflineAccountFormProps
> = (props) => {
  const [local, others] = splitProps(props, ['onCreate', 'onCancel']);

  const [{ t }] = useTranslation();

  const [form, { Field, Form }] = createForm<CreateOfflineAccountFormValues>({
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
              class='flex flex-col gap-3'
              validationState={field.error ? 'invalid' : 'valid'}
            >
              <TextFieldLabel class='flex flex-col gap-3'>
                {t('common.username')}
                <TextFieldInput
                  autocomplete='off'
                  id='username'
                  required
                  type='text'
                  value={field.value}
                  {...props}
                />
              </TextFieldLabel>
              <TextFieldErrorMessage>
                {t(
                  `createOfflineAccount.${field.error as CreateOfflineAccountFormSchemaErrors}`,
                )?.toString()}
              </TextFieldErrorMessage>
            </TextField>
          )}
        </Field>

        <DialogFooter>
          <Button type='submit'>{t('common.create')}</Button>
          <Button onClick={handleCancel} variant={'secondary'}>
            {t('common.cancel')}
          </Button>
        </DialogFooter>
      </div>
    </Form>
  );
};
