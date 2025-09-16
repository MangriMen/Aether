import type { PartialValues } from '@modular-forms/solid';
import {
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';
import {
  HooksSettingsSchema,
  type HooksSettingsSchemaValuesInput,
  type HooksSettingsSchemaValuesOutput,
} from '../model';
import { useHooksSettingsForm, useResetHooksSettingsFormValues } from '../lib';
import { cn } from '@/shared/lib';
import { CombinedTextField, LabeledField } from '@/shared/ui';
import { useFieldOnChangeSync } from '@/widgets/instance-settings-dialog/lib';

export type HooksSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  initialValues: Accessor<
    PartialValues<HooksSettingsSchemaValuesInput> | undefined
  >;
  onChangePartial?: (values: Partial<HooksSettingsSchemaValuesOutput>) => void;
};

export const HooksSettingsForm: Component<HooksSettingsFormProps> = (props) => {
  const [local, others] = splitProps(props, [
    'initialValues',
    'onChangePartial',
    'class',
  ]);

  const [form, { Form, Field }] = useHooksSettingsForm();
  useResetHooksSettingsFormValues(form, () => local.initialValues());

  const updatePreLaunch = useFieldOnChangeSync(
    HooksSettingsSchema,
    form,
    'preLaunch',
    (value) => value,
    (value) => local.onChangePartial?.({ preLaunch: value }),
  );

  const updateWrapper = useFieldOnChangeSync(
    HooksSettingsSchema,
    form,
    'wrapper',
    (value) => value,
    (value) => local.onChangePartial?.({ wrapper: value }),
  );

  const updatePostExit = useFieldOnChangeSync(
    HooksSettingsSchema,
    form,
    'postExit',
    (value) => value,
    (value) => local.onChangePartial?.({ postExit: value }),
  );

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <span class='text-lg font-medium'>Hooks</span>
      <LabeledField label='Pre launch'>
        <Field name='preLaunch' type='string'>
          {(field, inputProps) => (
            <CombinedTextField
              value={field.value ?? ''}
              label='Ran before the instance is launched'
              labelProps={{ variant: 'description' }}
              inputProps={{
                ...inputProps,
                type: 'text',
                placeholder: 'Enter pre-launch command',
                onBlur: (e) => {
                  inputProps.onBlur(e);
                  updatePreLaunch();
                },
              }}
            />
          )}
        </Field>
      </LabeledField>
      <LabeledField label='Wrapper'>
        <Field name='wrapper' type='string'>
          {(field, inputProps) => (
            <CombinedTextField
              value={field.value ?? ''}
              label='Wrapper command for launching Minecraft'
              labelProps={{ variant: 'description' }}
              inputProps={{
                ...inputProps,
                type: 'text',
                placeholder: 'Enter wrapper command',
                onBlur: (e) => {
                  inputProps.onBlur(e);
                  updateWrapper();
                },
              }}
            />
          )}
        </Field>
      </LabeledField>
      <LabeledField label='Post exit'>
        <Field name='postExit' type='string'>
          {(field, inputProps) => (
            <CombinedTextField
              value={field.value ?? ''}
              label='Ran after the game closes'
              labelProps={{ variant: 'description' }}
              inputProps={{
                ...inputProps,
                type: 'text',
                placeholder: 'Enter post-exit command',
                onBlur: (e) => {
                  inputProps.onBlur(e);
                  updatePostExit();
                },
              }}
            />
          )}
        </Field>
      </LabeledField>
    </Form>
  );
};
