import { setInput, validate, Field, type FormStore } from '@formisch/solid';
import { type Accessor, type Component } from 'solid-js';

import { OverridableEnvVarsField } from '@/entities/settings';
import { parseI18nError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { getDisplayValue } from '../lib';
import {
  isEnvVarsError,
  type JavaAndMemorySettingsSchema,
  type JavaAndMemorySettingsSchemaInput,
  type JavaAndMemorySettingsSchemaOutput,
} from '../model';

type Props = {
  form: FormStore<typeof JavaAndMemorySettingsSchema>;
  overridable?: boolean;
  defaultValues?: Accessor<
    Partial<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  onChangePartial: (values: Partial<JavaAndMemorySettingsSchemaOutput>) => void;
};

export const EnvVarsFieldSection: Component<Props> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Field of={props.form} path={['overrideEnvVars']}>
      {(overrideEnvVars) => (
        <Field of={props.form} path={['envVars']}>
          {(field) => (
            <OverridableEnvVarsField
              overridable={
                props.overridable && overrideEnvVars.input !== undefined
              }
              isOverridden={overrideEnvVars.input as boolean | undefined}
              value={getDisplayValue(
                props.overridable,
                field.input as string | undefined,
                overrideEnvVars.input as boolean | undefined,
                props.defaultValues?.()?.envVars ?? '',
              )}
              onOverrideChange={(value) => {
                setInput(props.form, {
                  path: ['overrideEnvVars'],
                  input: value,
                });
                props.onChangePartial({ overrideEnvVars: value });
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

                if (!isEnvVarsError(parsed.key)) {
                  return error;
                }

                return t(
                  `settings.environmentVariablesError.${parsed.key}`,
                  parsed.values,
                );
              })()}
              inputProps={{
                ...field.props,
                type: 'text',
                onBlur: async (e) => {
                  field.props.onBlur?.(e);
                  const result = await validate(props.form);

                  if (result.success) {
                    props.onChangePartial({
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
  );
};
