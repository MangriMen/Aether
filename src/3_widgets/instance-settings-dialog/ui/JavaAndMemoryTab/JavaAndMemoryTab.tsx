import type { Component } from 'solid-js';

import { setValue } from '@modular-forms/solid';
import { onCleanup, splitProps } from 'solid-js';

import type { EditInstance } from '@/entities/instances';

import { useEditInstance } from '@/entities/instances';
import { cn, debounce } from '@/shared/lib';

import {
  stringToEnvVars,
  stringToExtraLaunchArgs,
  useFieldOnChangeSync,
} from '../../lib';
import {
  useJavaAndMemoryForm,
  useResetJavaAndMemoryFormValues,
} from '../../lib/useJavaAndMemoryForm';
import {
  type InstanceSettingsTabProps,
  JavaAndMemorySettingsSchema,
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  MemoryMaximumSchema,
} from '../../model';
import { CustomEnvVarsField } from './CustomEnvVarsField';
import { ExtraLaunchArgsField } from './ExtraLaunchArgsField';
import { MemoryField } from './MemoryField';

export type JavaAndMemoryTabProps = {
  class?: string;
} & InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'settings', 'class']);

  const [form, { Field, Form }] = useJavaAndMemoryForm();

  useResetJavaAndMemoryFormValues(form, () => local.instance);

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceSimple = (edit: EditInstance) =>
    editInstance({ edit, id: local.instance.id });

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
            defaultValue={local.settings?.memory.maximum}
            onChange={(value) => {
              setValue(form, 'memory.maximum', value);
              updateMemory();
            }}
            value={field.value ?? null}
          />
        )}
      </Field>
      <Field name='extraLaunchArgs' type='string'>
        {(field, inputProps) => (
          <ExtraLaunchArgsField
            inputProps={{
              type: 'text',
              ...inputProps,
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateExtraLaunchArgs();
              },
            }}
            onIsCustomChange={(value) => {
              setValue(form, 'extraLaunchArgs', value);
              updateExtraLaunchArgs();
            }}
            value={field.value}
          />
        )}
      </Field>
      <Field name='customEnvVars' type='string'>
        {(field, inputProps) => (
          <CustomEnvVarsField
            inputProps={{
              type: 'text',
              ...inputProps,
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateEnvVars();
              },
            }}
            onChange={(value) => setValue(form, 'customEnvVars', value)}
            onIsCustomChange={(value) => {
              setValue(form, 'customEnvVars', value);
              updateEnvVars();
            }}
            value={field.value}
          />
        )}
      </Field>
    </Form>
  );
};
