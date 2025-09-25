import { getValues, type PartialValues } from '@modular-forms/solid';
import {
  createMemo,
  Show,
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { OverrideCheckbox } from '@/entities/settings';
import { cn, isNil, useFieldOnChangeSync } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField, LabeledField } from '@/shared/ui';

import { useHooksSettingsForm, useResetHooksSettingsFormValues } from '../lib';
import {
  HooksSettingsSchema,
  type HooksSettingsSchemaInput,
  type HooksSettingsSchemaOutput,
} from '../model';

export type HooksSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<PartialValues<HooksSettingsSchemaInput> | undefined>;
  onChangePartial?: (values: Partial<HooksSettingsSchemaOutput>) => void;
};

export const HooksSettingsForm: Component<HooksSettingsFormProps> = (props) => {
  const [local, others] = splitProps(props, [
    'overridable',
    'initialValues',
    'onChangePartial',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [form, { Form, Field }] = useHooksSettingsForm();
  useResetHooksSettingsFormValues(form, () => local.initialValues());

  const isOverride = createMemo(() => {
    if (!local.overridable) {
      return true;
    }

    const values = local.initialValues();

    if (!values) {
      return false;
    }

    const { preLaunch, wrapper, postExit } = values;

    return !(isNil(preLaunch) && isNil(wrapper) && isNil(postExit));
  });

  const handleOverrideChange = async (value: boolean) => {
    if (value) {
      const raw = getValues(form, { shouldValid: true });
      const parsed = HooksSettingsSchema.safeParse(raw);

      if (!parsed.success) {
        return;
      }

      local.onChangePartial?.({
        preLaunch: parsed.data.preLaunch ?? '',
        wrapper: parsed.data.wrapper ?? '',
        postExit: parsed.data.postExit ?? '',
      });
    } else {
      local.onChangePartial?.({
        preLaunch: null,
        wrapper: null,
        postExit: null,
      });
    }
  };

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
      <h2 class='text-lg font-medium'>Hooks</h2>
      <Show when={local.overridable}>
        <OverrideCheckbox
          class='mb-1'
          label={t('instanceSettings.customHooksSettings')}
          enabledValue={() => true}
          disabledValue={() => false}
          checked={isOverride()}
          onOverrideChange={handleOverrideChange}
        />
      </Show>
      <LabeledField label='Pre launch'>
        <Field name='preLaunch' type='string'>
          {(field, inputProps) => (
            <CombinedTextField
              disabled={!isOverride()}
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
              disabled={!isOverride()}
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
              disabled={!isOverride()}
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
