import { Field, getInput, type FormStore } from '@formisch/solid';
import { type Accessor, type Component } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { CombinedTextField, LabeledField } from '@/shared/ui';

import type {
  HookFieldConfig,
  HooksSettingsSchemaInput,
  HooksSettingsSchemaOutput,
} from '../model';
import type { HooksSettingsSchema } from '../model';

import { getHookDisplayValue } from '../lib';

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
            labelProps={{ variant: 'description' }}
            inputProps={{
              ...field.props,
              type: 'text',
              placeholder: t(
                'instanceSettings.hookSettings.enterCommandPlaceholder',
              ),
              onBlur: (e) => {
                field.props.onBlur?.(e);
                props.onChangePartial?.({
                  [props.fieldConfig.key]: getInput(props.form, {
                    path: [props.fieldConfig.key],
                  }),
                });
              },
            }}
          />
        )}
      </Field>
    </LabeledField>
  );
};
