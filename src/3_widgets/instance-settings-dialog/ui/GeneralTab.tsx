import type { Component } from 'solid-js';
import { createEffect, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { CombinedTextField } from '@/shared/ui';

import { type EditInstance, useEditInstance } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

import { createForm, reset, setValues, zodForm } from '@modular-forms/solid';
import { GeneralSettingsSchema, type InstanceSettingsTabProps } from '../model';
import { useFieldOnChangeSync } from '../lib';

export type GeneralTabProps = InstanceSettingsTabProps & { class?: string };

export const GeneralTab: Component<GeneralTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslation();

  const [form, { Form, Field }] = createForm({
    validate: zodForm(GeneralSettingsSchema),
    validateOn: 'blur',
    revalidateOn: 'blur',
  });

  createEffect(() => {
    setValues(form, { name: local.instance.name });
  });

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceSimple = (edit: EditInstance) =>
    editInstance({ id: local.instance.id, edit });

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
            label={t('common.name')}
            value={field.value ?? ''}
            errorMessage={field.error}
            inputProps={{
              type: 'text',
              ...inputProps,
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateMemory();
              },
            }}
          />
        )}
      </Field>
    </Form>
  );
};
