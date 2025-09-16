import type { Component } from 'solid-js';

import { createMemo, onCleanup, splitProps } from 'solid-js';

import type { EditInstance } from '@/entities/instances';

import { useEditInstance } from '@/entities/instances';
import {
  OverridableEnvVarsField,
  OverridableExtraLaunchArgsField,
  OverridableMemoryField,
} from '@/entities/settings';
import { cn, debounce } from '@/shared/lib';

import type { InstanceSettingsTabProps } from '../model';

import {
  stringToEnvVars,
  stringToExtraLaunchArgs,
  useFieldOnChangeSync,
  useJavaAndMemoryForm,
  useResetJavaAndMemoryFormValues,
} from '../lib';
import {
  JavaAndMemorySettingsSchema,
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  MemoryMaximumSchema,
  instanceSettingsToJavaAndMemorySettingsValues,
} from '../model';

export type JavaAndMemoryTabProps = InstanceSettingsTabProps & {
  class?: string;
};

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'globalSettings',
    'class',
  ]);

  const [form, { Form, Field }] = useJavaAndMemoryForm();

  const javaAndMemorySettingsFormValues = createMemo(() =>
    instanceSettingsToJavaAndMemorySettingsValues(local.instance),
  );

  useResetJavaAndMemoryFormValues(form, () =>
    javaAndMemorySettingsFormValues(),
  );

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
    (value) => editInstanceSimple(value),
  );

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Field name='memory.maximum' type='number'>
        {(field) => (
          <OverridableMemoryField
            overridable
            value={field.value ?? null}
            defaultValue={local.globalSettings?.memory.maximum}
            onChange={(value) => {
              updateMemory(value);
            }}
          />
        )}
      </Field>
      <Field name='extraLaunchArgs' type='string'>
        {(field, inputProps) => (
          <OverridableExtraLaunchArgsField
            overridable
            value={field.value}
            onOverrideChange={(value) => {
              updateExtraLaunchArgs(value);
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
          <OverridableEnvVarsField
            overridable
            value={field.value}
            onOverrideChange={(value) => {
              updateEnvVars(value);
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
