import type { Component } from 'solid-js';
import { createMemo, onCleanup, splitProps } from 'solid-js';

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
import {
  useJavaAndMemoryForm,
  useResetJavaAndMemoryFormValues,
} from '../../lib/useJavaAndMemoryForm';
import { OverridableExtraLaunchArgsField } from '../../../../5_entities/settings/ui/OverridableExtraLaunchArgsField';
import { instanceToJavaAndMemorySettingsValues } from '../../model/converter';
import {
  OverridableMemoryField,
  OverridableEnvVarsField,
} from '@/entities/settings';

export type JavaAndMemoryTabProps = {
  class?: string;
} & InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'globalSettings',
    'class',
  ]);

  const [form, { Form, Field }] = useJavaAndMemoryForm();

  const javaAndMemorySettingsFormValues = createMemo(() =>
    instanceToJavaAndMemorySettingsValues(local.instance),
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
