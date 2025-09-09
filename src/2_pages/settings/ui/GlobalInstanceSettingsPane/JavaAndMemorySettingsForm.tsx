import { cn, debounce } from '@/shared/lib';
import type {
  JavaAndMemorySettingsSchemaValuesInput,
  JavaAndMemorySettingsSchemaValuesOutput,
} from '@/widgets/instance-settings-dialog';
import {
  JavaAndMemorySettingsSchema,
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  MemoryMaximumSchema,
} from '@/widgets/instance-settings-dialog';
import { useFieldOnChangeSync } from '@/widgets/instance-settings-dialog/lib';
import {
  useJavaAndMemoryFormRequired,
  useResetJavaAndMemoryFormValues,
} from '@/widgets/instance-settings-dialog/lib/useJavaAndMemoryForm';
import { CustomEnvVarsField } from '@/widgets/instance-settings-dialog/ui/JavaAndMemoryTab/CustomEnvVarsField';
import { ExtraLaunchArgsField } from '@/widgets/instance-settings-dialog/ui/JavaAndMemoryTab/ExtraLaunchArgsField';
import { MemoryField } from '@/widgets/instance-settings-dialog/ui/JavaAndMemoryTab/MemoryField';
import { setValue } from '@modular-forms/solid';
import {
  type Accessor,
  onCleanup,
  splitProps,
  type Component,
  type ComponentProps,
  createMemo,
} from 'solid-js';

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

  useResetJavaAndMemoryFormValues(form, local.initialValues);

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
    (value) => ({
      memory: { maximum: value },
    }),
    (value) => {
      if (value.memory.maximum === null || value.memory.maximum === undefined) {
        return;
      }

      onChangePartialDebounced()({
        memory: { maximum: value.memory.maximum },
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
    async (value, formValue) => {
      local.onChangePartial({ customEnvVars: value });
      setValue(form, 'customEnvVars', formValue);
    },
  );

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Field name='memory.maximum' type='number'>
        {(field) => (
          <MemoryField
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
