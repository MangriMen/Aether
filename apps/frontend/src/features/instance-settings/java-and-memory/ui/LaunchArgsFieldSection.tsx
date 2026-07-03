import type { FormStore } from '@formisch/solid';

import { setInput, getInput, Field } from '@formisch/solid';
import { type Accessor, type Component } from 'solid-js';

import { OverridableLaunchArgsField } from '@/entities/settings';

import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '../model';
import type { JavaAndMemorySettingsSchema } from '../model';

import { getDisplayValue } from '../lib';

type Props = {
  form: FormStore<typeof JavaAndMemorySettingsSchema>;
  overridable?: boolean;
  defaultValues?: Accessor<
    Partial<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  onChangePartial: (values: Partial<JavaAndMemorySettingsSchemaOutput>) => void;
};

export const LaunchArgsFieldSection: Component<Props> = (props) => {
  return (
    <Field of={props.form} path={['overrideLaunchArgs']}>
      {(overrideLaunchArgs) => (
        <Field of={props.form} path={['launchArgs']}>
          {(field) => (
            <OverridableLaunchArgsField
              overridable={
                props.overridable && overrideLaunchArgs.input !== undefined
              }
              isOverridden={overrideLaunchArgs.input as boolean | undefined}
              value={getDisplayValue(
                props.overridable,
                field.input as string | undefined,
                overrideLaunchArgs.input as boolean | undefined,
                props.defaultValues?.()?.launchArgs ?? '',
              )}
              onOverrideChange={(value) => {
                setInput(props.form, {
                  path: ['overrideLaunchArgs'],
                  input: value,
                });
                props.onChangePartial({ overrideLaunchArgs: value });
              }}
              inputProps={{
                ...field.props,
                type: 'text',
                onBlur: (e) => {
                  field.props.onBlur?.(e);
                  props.onChangePartial({
                    launchArgs: getInput(props.form, { path: ['launchArgs'] }),
                  });
                },
              }}
            />
          )}
        </Field>
      )}
    </Field>
  );
};
