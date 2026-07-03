import type { Component, ComponentProps } from 'solid-js';

import { createForm, reset, setErrors, Form, Field } from '@formisch/solid';
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
  externalErrors?: Record<string, string>;
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

  const form = createForm({ schema: CreateOfflineAccountFormSchema });

  const handleSubmit = (values: CreateOfflineAccountFormValues) => {
    local.onCreate(values.username);
  };

  const handleCancel = () => {
    reset(form);
    local.onCancel();
  };

  createEffect(() => {
    const errors = local.externalErrors;
    untrack(() => {
      if (errors?.username) {
        setErrors(form, { path: ['username'], errors: [errors.username] });
      }
    });
  });

  const formatError = (error: string | undefined) => {
    if (!error) {
      return;
    }

    if (isCreateOfflineAccountFormSchemaError(error)) {
      return t(`errors.auth.${error}`, createOfflineAccountErrorParams[error]);
    }

    return error;
  };

  return (
    <Form of={form} onSubmit={handleSubmit}>
      <div class='gap-4 flex flex-col' {...others}>
        <Field of={form} path={['username']}>
          {(field) => (
            <CombinedTextField
              value={field.input as string}
              label={t('common.username')}
              inputProps={{ type: 'text', ...field.props }}
              errorMessage={formatError(field.errors?.[0])}
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
