import {
  Field,
  getErrors,
  getInput,
  setInput,
  validate,
  type FormStore,
} from '@formisch/solid';
import { type Accessor, type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedTextField, LabeledField } from '@/shared/ui-kit';

import type { HookFieldConfig } from '../model/hooksFields';
import type {
  HooksSettingsSchema,
  HooksSettingsSchemaInput,
  HooksSettingsSchemaOutput,
} from '../model/hooksSettingsSchema';

import { getHookDisplayValue } from '../model/hooksDisplay';
import { HOOK_COMMAND_ERRORS } from '../model/hooksSettingsSchema';
import { normalizeHookCommand } from '../model/parseCommandLine';

export type HookTextFieldProps = {
  fieldConfig: HookFieldConfig;
  form: FormStore<typeof HooksSettingsSchema>;
  isDisabled: () => boolean;
  overrideHooksInput: boolean | undefined;
  onChangePartial?: (values: Partial<HooksSettingsSchemaOutput>) => void;
  defaultValues?: Accessor<Partial<HooksSettingsSchemaInput> | undefined>;
  overridable?: boolean;
};

export const HookTextField: Component<HookTextFieldProps> = (props) => {
  const [{ t }] = useTranslation();

  const translateHookError = (message: string | undefined) => {
    const key = Object.values(HOOK_COMMAND_ERRORS).find(
      (error) => error === message,
    );

    return key ? t(key) : message;
  };

  return (
    <LabeledField label={t(props.fieldConfig.labelKey)}>
      <Field of={props.form} path={[props.fieldConfig.key]}>
        {(field) => (
          <CombinedTextField
            disabled={props.isDisabled()}
            value={getHookDisplayValue(
              props.defaultValues,
              props.overridable,
              props.fieldConfig.key,
              field.input as string | undefined,
              props.overrideHooksInput,
            )}
            label={t(props.fieldConfig.subLabelKey)}
            errorMessage={translateHookError(field.errors?.[0])}
            labelProps={{ variant: 'description' }}
            inputProps={{
              ...field.props,
              type: 'text',
              placeholder: t(
                'instanceSettings.hookSettings.enterCommandPlaceholder',
              ),
              onBlur: async () => {
                field.props.onBlur?.();
                await validate(props.form);

                // A command the core cannot read is not saved; the field shows why.
                if (getErrors(props.form, { path: [props.fieldConfig.key] })) {
                  return;
                }

                const path = [props.fieldConfig.key] as const;
                const command = normalizeHookCommand(
                  getInput(props.form, { path }) ?? '',
                );

                // A refresh keeps the field as typed when the stored command is unchanged, so
                // the field shows the trimmed command that is saved.
                setInput(props.form, { path, input: command });
                props.onChangePartial?.({ [props.fieldConfig.key]: command });
              },
            }}
          />
        )}
      </Field>
    </LabeledField>
  );
};
