import type { FormErrors, SubmitHandler } from '@modular-forms/solid';
import type { Component, ComponentProps } from 'solid-js';

import { createForm, reset, setError, zodForm } from '@modular-forms/solid';
import { createEffect, splitProps, untrack } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { Button, CombinedTextField, DialogFooter } from '@/shared/ui';

import type { CreateOfflineAccountFormValues } from '../model';

import {
  createOfflineAccountErrorParams,
  CreateOfflineAccountFormSchema,
  isCreateOfflineAccountFormSchemaError,
} from '../model';

export type CreateOfflineAccountFormProps = ComponentProps<'div'> & {
  onCreate: (username: string) => void;
  onCancel: () => void;
  isLoading?: boolean;
  externalErrors?: FormErrors<CreateOfflineAccountFormValues>;
  onClearErrors?: () => void;
};

export const CreateOfflineAccountForm: Component<
  CreateOfflineAccountFormProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'onCreate',
    'onCancel',
    'isLoading',
    'externalErrors',
    'onClearErrors',
  ]);

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

  createEffect(() => {
    const errors = local.externalErrors;
    const isSubmitting = form.submitting;
    const isInvalid = form.invalid;
    untrack(() => {
      if (!isSubmitting && !isInvalid && errors?.username) {
        setError(form, 'username', errors.username);
      }
    });
  });

  const formatError = (error: string) => {
    if (!error) {
      return;
    }

    if (isCreateOfflineAccountFormSchemaError(error)) {
      return t(`errors.auth.${error}`, createOfflineAccountErrorParams[error]);
    }

    return error;
  };

  return (
    <Form onSubmit={handleSubmit}>
      <div class='flex flex-col gap-4' {...others}>
        <Field name='username'>
          {(field, props) => (
            <CombinedTextField
              value={field.value}
              label={t('common.username')}
              inputProps={{ type: 'text', ...props }}
              errorMessage={formatError(field.error)}
            />
          )}
        </Field>

        <DialogFooter>
          <Button
            type='submit'
            // Save focus on input element if it's invalid
            onMouseDown={(e) => e.preventDefault()}
            loading={local.isLoading}
          >
            {t('common.create')}
          </Button>
          <Button variant='secondary' onClick={handleCancel}>
            {t('common.cancel')}
          </Button>
        </DialogFooter>
      </div>
    </Form>
  );
};
