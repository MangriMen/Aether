import type { Component } from 'solid-js';
import { onCleanup, splitProps } from 'solid-js';

import { cn, debounce } from '@/shared/lib';

import type { EditInstance } from '@/entities/instances';
import { useEditInstance } from '@/entities/instances';

import {
  JavaAndMemorySettingsSchema,
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  MemoryMaximumSchema,
  type InstanceSettingsTabProps,
} from '../../model';
import {
  stringToEnvVars,
  stringToExtraLaunchArgs,
  useFieldOnChangeSync,
} from '../../lib';
import { MemoryField } from './MemoryField';
import {
  useJavaAndMemoryForm,
  useResetJavaAndMemoryFormValues,
} from '../../lib/useJavaAndMemoryForm';
import { ExtraLaunchArgsField } from './ExtraLaunchArgsField';
import { CustomEnvVarsField } from './CustomEnvVarsField';
import { setValue } from '@modular-forms/solid';

export type JavaAndMemoryTabProps = {
  class?: string;
} & InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'settings', 'class']);

  const [form, { Form, Field }] = useJavaAndMemoryForm();

  useResetJavaAndMemoryFormValues(form, () => local.instance);

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceSimple = (edit: EditInstance) =>
    editInstance({ id: local.instance.id, edit });

  // eslint-disable-next-line solid/reactivity
  const editInstanceSimpleDebounced = debounce(
    editInstanceSimple,
    MEMORY_SLIDER_HANDLE_DEBOUNCE,
  );
  onCleanup(() => {
    editInstanceSimpleDebounced.callAndClear();
  });

  const updateMemory = useFieldOnChangeSync(
    MemoryMaximumSchema,
    form,
    'memory.maximum',
    (value) => ({
      memory: value ? { maximum: value } : null,
    }),
    editInstanceSimpleDebounced,
  );

  const updateExtraLaunchArgs = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'extraLaunchArgs',
    (value) => ({
      extraLaunchArgs: value ? stringToExtraLaunchArgs(value) : null,
    }),
    editInstanceSimple,
  );

  const updateEnvVars = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'customEnvVars',
    (value) => ({
      customEnvVars: value ? stringToEnvVars(value) : null,
    }),
    async (value, formValue) => {
      await editInstanceSimple(value);
      setValue(form, 'customEnvVars', formValue);
    },
  );

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Field name='memory.maximum' type='number'>
        {(field) => (
          <MemoryField
            value={field.value ?? null}
            defaultValue={local.settings?.memory.maximum}
            onChange={(value) => {
              setValue(form, 'memory.maximum', value);
              updateMemory();
            }}
          />
        )}
      </Field>
      <Field name='extraLaunchArgs' type='string'>
        {(field, inputProps) => (
          <ExtraLaunchArgsField
            value={field.value}
            onIsCustomChange={(value) => {
              setValue(form, 'extraLaunchArgs', value);
              updateExtraLaunchArgs();
            }}
            inputProps={{
              type: 'text',
              ...inputProps,
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateExtraLaunchArgs();
              },
            }}
          />
        )}
      </Field>
      <Field name='customEnvVars' type='string'>
        {(field, inputProps) => (
          <CustomEnvVarsField
            value={field.value}
            onChange={(value) => setValue(form, 'customEnvVars', value)}
            onIsCustomChange={(value) => {
              setValue(form, 'customEnvVars', value);
              updateEnvVars();
            }}
            inputProps={{
              type: 'text',
              ...inputProps,
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateEnvVars();
              },
            }}
          />
        )}
      </Field>
    </Form>
  );
};
