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

export type CreateOfflineAccountFormProps = ComponentProps<'div'> & {
  onCreate: (username: string) => void;
  onCancel: () => void;
};

export const CreateOfflineAccountForm: Component<
  CreateOfflineAccountFormProps
> = (props) => {
  const [local, others] = splitProps(props, ['onCreate', 'onCancel']);

  const [{ t }] = useTranslation();

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
              <TextFieldLabel class='flex flex-col gap-3'>
                {t('common.username')}
                <TextFieldInput
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
          <Button variant={'secondary'} onClick={handleCancel}>
            {t('common.cancel')}
          </Button>
        </DialogFooter>
      </div>
    </Form>
  );
};
