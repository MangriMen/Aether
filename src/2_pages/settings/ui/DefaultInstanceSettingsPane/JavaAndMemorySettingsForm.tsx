import { setValue } from '@modular-forms/solid';
import {
  type Accessor,
  onCleanup,
  splitProps,
  type Component,
  type ComponentProps,
  createMemo,
} from 'solid-js';

import type {
  JavaAndMemorySettingsSchemaValuesInput,
  JavaAndMemorySettingsSchemaValuesOutput,
} from '@/widgets/instance-settings-dialog';

import { OverridableEnvVarsField } from '@/entities/settings';
import { OverridableExtraLaunchArgsField } from '@/entities/settings/ui/OverridableExtraLaunchArgsField';
import { OverridableMemoryField } from '@/entities/settings/ui/OverridableMemoryField';
import { cn, debounce } from '@/shared/lib';
import {
  JavaAndMemorySettingsSchema,
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  MemoryMaximumSchema,
} from '@/widgets/instance-settings-dialog';
import { useFieldOnChangeSync } from '@/widgets/instance-settings-dialog/lib';
import {
  useJavaAndMemoryFormRequired,
  useResetJavaAndMemoryFormRequiredValues,
} from '@/widgets/instance-settings-dialog/lib/useJavaAndMemoryForm';

export type JavaAndMemorySettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  initialValues: Accessor<
    Partial<JavaAndMemorySettingsSchemaValuesInput> | undefined
  >;
  onChangePartial: (
    values: Partial<JavaAndMemorySettingsSchemaValuesOutput>,
  ) => void;
};

export const JavaAndMemorySettingsForm: Component<
  JavaAndMemorySettingsFormProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'initialValues',
    'onChangePartial',
    'class',
  ]);

  const [form, { Form, Field }] = useJavaAndMemoryFormRequired();

  useResetJavaAndMemoryFormRequiredValues(form, local.initialValues);

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

  const updateExtraLaunchArgs = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'extraLaunchArgs',
    (value) => value,
    (value) => {
      local.onChangePartial({
        extraLaunchArgs: value,
      });
    },
  );

  const updateEnvVars = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'customEnvVars',
    (value) => value,
    (value) => {
      local.onChangePartial({ customEnvVars: value });
    },
  );

  return (
    <Form class={cn('flex flex-col gap-4', local.class)} {...others}>
      <Field name='memory.maximum' type='number'>
        {(field) => (
          <OverridableMemoryField
            value={field.value ?? null}
            defaultValue={local.initialValues()?.memory?.maximum ?? undefined}
            onChange={(value) => {
              setValue(form, 'memory.maximum', value);
              updateMemory();
            }}
          />
        )}
      </Field>
      <Field name='extraLaunchArgs' type='string'>
        {(field, inputProps) => (
          <OverridableExtraLaunchArgsField
            value={field.value}
            inputProps={{
              ...inputProps,
              type: 'text',
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
            value={field.value}
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
