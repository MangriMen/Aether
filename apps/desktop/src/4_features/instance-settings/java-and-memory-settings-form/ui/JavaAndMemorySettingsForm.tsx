import type { Accessor, Component, ComponentProps } from 'solid-js';

import { setValue } from '@modular-forms/solid';
import { onCleanup, splitProps, createMemo } from 'solid-js';

import { OverridableEnvVarsField } from '@/entities/settings';
import { OverridableLaunchArgsField } from '@/entities/settings';
import { OverridableMemoryField } from '@/entities/settings/ui/OverridableMemoryField';
import { cn, debounce, useFieldOnChangeSync } from '@/shared/lib';

import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '../model';

import {
  useJavaAndMemorySettingsForm,
  useResetJavaAndMemorySettingsForm,
} from '../lib';
import {
  JavaAndMemorySettingsSchema,
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  MemoryMaximumSchema,
} from '../model';

export type JavaAndMemorySettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<
    Partial<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  defaultValues?: Accessor<
    Partial<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  onChangePartial: (values: Partial<JavaAndMemorySettingsSchemaOutput>) => void;
};

export const JavaAndMemorySettingsForm: Component<
  JavaAndMemorySettingsFormProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'overridable',
    'initialValues',
    'defaultValues',
    'onChangePartial',
    'class',
  ]);

  const [form, { Form, Field }] = useJavaAndMemorySettingsForm();
  useResetJavaAndMemorySettingsForm(form, local.initialValues);

  const onChangePartialDebounced = createMemo(() =>
    debounce(local.onChangePartial, MEMORY_SLIDER_HANDLE_DEBOUNCE),
  );
  onCleanup(() => {
    onChangePartialDebounced().callAndClear();
  });

  const updateMemory = useFieldOnChangeSync(
    MemoryMaximumSchema,
    form,
    'memory.maximum',
    (value) => value,
    (value) => {
      onChangePartialDebounced()({
        memory: { maximum: value },
      });
    },
  );

  const updateLaunchArgs = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'launchArgs',
    (value) => value,
    (value) => {
      local.onChangePartial({
        launchArgs: value,
      });
    },
  );

  const updateEnvVars = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'envVars',
    (value) => value,
    (value) => {
      local.onChangePartial({ envVars: value });
    },
  );

  return (
    <Form class={cn('flex flex-col gap-4', local.class)} {...others}>
      <Field name='memory.maximum' type='number'>
        {(field) => (
          <OverridableMemoryField
            overridable={local.overridable}
            value={field.value ?? null}
            defaultValue={local.defaultValues?.()?.memory?.maximum ?? undefined}
            onChange={(value) => {
              setValue(form, 'memory.maximum', value);
              updateMemory();
            }}
          />
        )}
      </Field>
      <Field name='launchArgs' type='string'>
        {(field, inputProps) => (
          <OverridableLaunchArgsField
            overridable={local.overridable}
            value={field.value}
            onOverrideChange={updateLaunchArgs}
            inputProps={{
              ...inputProps,
              type: 'text',
              onBlur: (e) => {
                inputProps.onBlur(e);
                updateLaunchArgs();
              },
            }}
          />
        )}
      </Field>
      <Field name='envVars' type='string'>
        {(field, inputProps) => (
          <OverridableEnvVarsField
            overridable={local.overridable}
            value={field.value}
            onOverrideChange={updateEnvVars}
            inputProps={{
              ...inputProps,
              type: 'text',
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
