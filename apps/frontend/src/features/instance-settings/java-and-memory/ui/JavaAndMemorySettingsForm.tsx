import type { Accessor, Component, ComponentProps } from 'solid-js';

import { setInput, getInput, validate, Form, Field } from '@formisch/solid';
import { onCleanup, splitProps, createMemo } from 'solid-js';

import { OverridableEnvVarsField } from '@/entities/settings';
import { OverridableLaunchArgsField } from '@/entities/settings';
import { OverridableMemoryField } from '@/entities/settings';
import { cn, debounce, parseI18nError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '../model';

import {
  useJavaAndMemorySettingsForm,
  useResetJavaAndMemorySettingsForm,
} from '../lib';
import { isEnvVarsError, MEMORY_SLIDER_HANDLE_DEBOUNCE } from '../model';

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

  const [{ t }] = useTranslation();

  const form = useJavaAndMemorySettingsForm();
  useResetJavaAndMemorySettingsForm(form, local.initialValues);

  const onChangePartialDebounced = createMemo(() =>
    debounce(local.onChangePartial, MEMORY_SLIDER_HANDLE_DEBOUNCE),
  );
  onCleanup(() => {
    onChangePartialDebounced().callAndClear();
  });

  const getMemoryDisplayValue = (
    value: number | undefined,
    isOverridden: boolean | undefined,
  ) => {
    const defaultMemory = local.defaultValues?.()?.memory?.maximum;

    if (local.overridable && !isOverridden) {
      return defaultMemory;
    }

    return value ?? defaultMemory;
  };

  const getLaunchArgsDisplayValue = (
    value: string | undefined,
    isOverridden: boolean | undefined,
  ) => {
    const defaultLaunchArgs = local.defaultValues?.()?.launchArgs ?? '';

    if (local.overridable && !isOverridden) {
      return defaultLaunchArgs;
    }

    return value ?? defaultLaunchArgs;
  };

  const getEnvVarsDisplayValue = (
    value: string | undefined,
    isOverridden: boolean | undefined,
  ) => {
    const defaultEnvVars = local.defaultValues?.()?.envVars ?? '';

    if (local.overridable && !isOverridden) {
      return defaultEnvVars;
    }

    return value ?? defaultEnvVars;
  };

  return (
    <Form
      of={form}
      class={cn('gap-4 flex flex-col', local.class)}
      onSubmit={() => {}}
      {...others}
    >
      <Field of={form} path={['overrideMemory']}>
        {(overrideMemory) => (
          <Field of={form} path={['memory', 'maximum']}>
            {(field) => (
              <OverridableMemoryField
                overridable={
                  local.overridable && overrideMemory.input !== undefined
                }
                isOverridden={overrideMemory.input as boolean | undefined}
                value={getMemoryDisplayValue(
                  field.input as number | undefined,
                  overrideMemory.input as boolean | undefined,
                )}
                onChange={(value: number) => {
                  setInput(form, { path: ['memory', 'maximum'], input: value });
                  onChangePartialDebounced()({
                    memory: { maximum: value },
                  });
                }}
                onOverrideChange={(value) => {
                  setInput(form, { path: ['overrideMemory'], input: value });
                  local.onChangePartial({ overrideMemory: value });
                }}
              />
            )}
          </Field>
        )}
      </Field>
      <Field of={form} path={['overrideLaunchArgs']}>
        {(overrideLaunchArgs) => (
          <Field of={form} path={['launchArgs']}>
            {(field) => (
              <OverridableLaunchArgsField
                overridable={
                  local.overridable && overrideLaunchArgs.input !== undefined
                }
                isOverridden={overrideLaunchArgs.input}
                value={getLaunchArgsDisplayValue(
                  field.input as string | undefined,
                  overrideLaunchArgs.input as boolean | undefined,
                )}
                onOverrideChange={(value) => {
                  setInput(form, {
                    path: ['overrideLaunchArgs'],
                    input: value,
                  });
                  local.onChangePartial({ overrideLaunchArgs: value });
                }}
                inputProps={{
                  ...field.props,
                  type: 'text',
                  onBlur: (e) => {
                    field.props.onBlur?.(e);
                    local.onChangePartial({
                      launchArgs: getInput(form, { path: ['launchArgs'] }),
                    });
                  },
                }}
              />
            )}
          </Field>
        )}
      </Field>
      <Field of={form} path={['overrideEnvVars']}>
        {(overrideEnvVars) => (
          <Field of={form} path={['envVars']}>
            {(field) => (
              <OverridableEnvVarsField
                overridable={
                  local.overridable && overrideEnvVars.input !== undefined
                }
                isOverridden={overrideEnvVars.input}
                value={getEnvVarsDisplayValue(
                  field.input as string | undefined,
                  overrideEnvVars.input as boolean | undefined,
                )}
                onOverrideChange={(value) => {
                  setInput(form, { path: ['overrideEnvVars'], input: value });
                  local.onChangePartial({ overrideEnvVars: value });
                }}
                errorMessage={(() => {
                  const error = field.errors?.[0];

                  if (!error) {
                    return undefined;
                  }

                  const parsed = parseI18nError(error);

                  if (!parsed) {
                    return error;
                  }

                  return isEnvVarsError(parsed.key)
                    ? t(
                        `settings.environmentVariablesError.${parsed.key}`,
                        parsed.values,
                      )
                    : error;
                })()}
                inputProps={{
                  ...field.props,
                  type: 'text',
                  onBlur: async (e) => {
                    field.props.onBlur?.(e);
                    const result = await validate(form);

                    if (result.success) {
                      local.onChangePartial({
                        envVars: result.output.envVars,
                      });
                    }
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
