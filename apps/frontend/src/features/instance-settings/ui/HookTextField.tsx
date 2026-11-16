import {
  Field,
  getErrors,
  getInput,
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
import { UNCLOSED_QUOTE_ERROR } from '../model/hooksSettingsSchema';
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
            errorMessage={
              field.errors?.[0] === UNCLOSED_QUOTE_ERROR
                ? t(UNCLOSED_QUOTE_ERROR)
                : field.errors?.[0]
            }
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

                props.onChangePartial?.({
                  [props.fieldConfig.key]: normalizeHookCommand(
                    getInput(props.form, { path: [props.fieldConfig.key] }) ??
                      '',
                  ),
                });
              },
            }}
          />
        )}
      </Field>
    </LabeledField>
  );
};
