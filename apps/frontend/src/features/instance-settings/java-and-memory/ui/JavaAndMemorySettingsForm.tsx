import type { Maybe, PartialValues } from '@modular-forms/solid';
import type { Accessor, Component, ComponentProps } from 'solid-js';

import { setValue } from '@modular-forms/solid';
import { onCleanup, splitProps, createMemo } from 'solid-js';

import { OverridableEnvVarsField } from '@/entities/settings';
import { OverridableLaunchArgsField } from '@/entities/settings';
import { OverridableMemoryField } from '@/entities/settings';
import {
  cn,
  debounce,
  parseI18nError,
  useFieldOnChangeSync,
} from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '../model';

import {
  useJavaAndMemorySettingsForm,
  useResetJavaAndMemorySettingsForm,
} from '../lib';
import {
  isEnvVarsError,
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
    PartialValues<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  defaultValues?: Accessor<
    PartialValues<JavaAndMemorySettingsSchemaInput> | undefined
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

  const [{ t }] = useTranslation();

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
    (maximum) => {
      onChangePartialDebounced()({
        memory: { maximum },
      });
    },
  );

  const updateLaunchArgs = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'launchArgs',
    (launchArgs) => {
      local.onChangePartial({
        launchArgs,
      });
    },
  );

  const updateEnvVars = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'envVars',
    (envVars) => local.onChangePartial({ envVars }),
  );

  const updateOverrideMemory = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'overrideMemory',
    (overrideMemory) => local.onChangePartial({ overrideMemory }),
  );

  const updateOverrideLaunchArgs = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'overrideLaunchArgs',
    (overrideLaunchArgs) => local.onChangePartial({ overrideLaunchArgs }),
  );

  const updateOverrideEnvVars = useFieldOnChangeSync(
    JavaAndMemorySettingsSchema,
    form,
    'overrideEnvVars',
    (overrideEnvVars) => local.onChangePartial({ overrideEnvVars }),
  );

  const getMemoryDisplayValue = (
    value: Maybe<number>,
    isOverridden: boolean | undefined,
  ) => {
    const defaultMemory = local.defaultValues?.()?.memory?.maximum ?? undefined;

    if (local.overridable && !isOverridden) {
      return defaultMemory;
    }

    return value ?? defaultMemory;
  };

  const getLaunchArgsDisplayValue = (
    value: Maybe<string>,
    isOverridden: boolean | undefined,
  ) => {
    const defaultLaunchArgs = local.defaultValues?.()?.launchArgs ?? '';

    if (local.overridable && !isOverridden) {
      return defaultLaunchArgs;
    }

    return value ?? defaultLaunchArgs;
  };

  const getEnvVarsDisplayValue = (
    value: Maybe<string>,
    isOverridden: boolean | undefined,
  ) => {
    const defaultMemory = local.defaultValues?.()?.envVars ?? '';

    if (local.overridable && !isOverridden) {
      return defaultMemory;
    }

    return value ?? defaultMemory;
  };

  return (
    <Form class={cn('gap-4 flex flex-col', local.class)} {...others}>
      <Field name='overrideMemory' type='boolean'>
        {(overrideMemory) => (
          <Field name='memory.maximum' type='number'>
            {(field) => (
              <OverridableMemoryField
                overridable={
                  local.overridable && overrideMemory.value !== undefined
                }
                isOverridden={overrideMemory.value}
                value={getMemoryDisplayValue(field.value, overrideMemory.value)}
                onChange={(value) => {
                  setValue(form, 'memory.maximum', value);
                  updateMemory();
                }}
                onOverrideChange={(value) => {
                  setValue(form, 'overrideMemory', value);
                  updateOverrideMemory();
                }}
              />
            )}
          </Field>
        )}
      </Field>
      <Field name='overrideLaunchArgs' type='boolean'>
        {(overrideLaunchArgs) => (
          <Field name='launchArgs' type='string'>
            {(field, inputProps) => (
              <OverridableLaunchArgsField
                overridable={
                  local.overridable && overrideLaunchArgs.value !== undefined
                }
                isOverridden={overrideLaunchArgs.value}
                value={getLaunchArgsDisplayValue(
                  field.value,
                  overrideLaunchArgs.value,
                )}
                onOverrideChange={(value) => {
                  setValue(form, 'overrideLaunchArgs', value);
                  updateOverrideLaunchArgs();
                }}
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
        )}
      </Field>
      <Field name='overrideEnvVars' type='boolean'>
        {(overrideEnvVars) => (
          <Field name='envVars' type='string'>
            {(field, inputProps) => (
              <OverridableEnvVarsField
                overridable={
                  local.overridable && overrideEnvVars.value !== undefined
                }
                isOverridden={overrideEnvVars.value}
                value={getEnvVarsDisplayValue(
                  field.value,
                  overrideEnvVars.value,
                )}
                onOverrideChange={(value) => {
                  setValue(form, 'overrideEnvVars', value);
                  updateOverrideEnvVars();
                }}
                errorMessage={(() => {
                  const parsed = parseI18nError(field.error);

                  if (!parsed) {
                    return field.error;
                  }

                  return isEnvVarsError(parsed.key)
                    ? t(
                        `settings.environmentVariablesError.${parsed.key}`,
                        parsed.values,
                      )
                    : field.error;
                })()}
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
        )}
      </Field>
    </Form>
  );
};
