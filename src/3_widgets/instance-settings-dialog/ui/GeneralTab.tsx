import type { Component } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn, useFieldOnChangeSync } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import {
  useGeneralSettingsForm,
  useGeneralSettingsHandler,
  useResetGeneralSettingsFormValues,
} from '../lib';
import { GeneralSettingsSchema, type InstanceSettingsTabProps } from '../model';

export type GeneralTabProps = InstanceSettingsTabProps & { class?: string };

export const GeneralTab: Component<GeneralTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'editInstance',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const { initialValues, onChange } = useGeneralSettingsHandler({
    instance: () => local.instance,
    editInstance: () => local.editInstance,
  });

  const [form, { Form, Field }] = useGeneralSettingsForm();
  useResetGeneralSettingsFormValues(form, initialValues);

  const updateName = useFieldOnChangeSync(
    GeneralSettingsSchema,
    form,
    'name',
    (value) => value,
    (value) => {
      onChange({
        name: value,
      });
    },
  );

  return (
    <Form class={cn('flex flex-col', local.class)} {...others}>
      <Field name='name' type='string'>
        {(field, inputProps) => (
          <CombinedTextField
            label={t('common.name')}
            value={field.value ?? ''}
            errorMessage={field.error}
            inputProps={{
              type: 'text',
              ...inputProps,
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateName();
              },
            }}
          />
        )}
      </Field>
    </Form>
  );
};
