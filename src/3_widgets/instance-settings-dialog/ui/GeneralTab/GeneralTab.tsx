import type { Component } from 'solid-js';

import { createForm, reset, setValues, zodForm } from '@modular-forms/solid';
import { createEffect, splitProps } from 'solid-js';

import { type EditInstance, useEditInstance } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField } from '@/shared/ui';

import { useFieldOnChangeSync } from '../../lib';
import {
  GeneralSettingsSchema,
  type InstanceSettingsTabProps,
} from '../../model';

export type GeneralTabProps = { class?: string } & InstanceSettingsTabProps;

export const GeneralTab: Component<GeneralTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const [form, { Field, Form }] = createForm({
    revalidateOn: 'blur',
    validate: zodForm(GeneralSettingsSchema),
    validateOn: 'blur',
  });

  createEffect(() => {
    setValues(form, { name: local.instance.name });
  });

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceSimple = (edit: EditInstance) =>
    editInstance({ edit, id: local.instance.id });

  const updateMemory = useFieldOnChangeSync(
    GeneralSettingsSchema,
    form,
    'name',
    (value) => ({ name: value }),
    async (value) => {
      try {
        await editInstanceSimple(value);
      } catch {
        reset(form, 'name', { initialValue: local.instance.name });
      }
    },
  );

  return (
    <Form class={cn('flex flex-col', local.class)} {...others}>
      <Field name='name' type='string'>
        {(field, inputProps) => (
          <CombinedTextField
            errorMessage={field.error}
            inputProps={{
              type: 'text',
              ...inputProps,
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateMemory();
              },
            }}
            label={t('common.name')}
            value={field.value ?? ''}
          />
        )}
      </Field>
    </Form>
  );
};
