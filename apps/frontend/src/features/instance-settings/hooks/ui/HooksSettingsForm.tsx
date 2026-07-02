import type { Maybe } from '@modular-forms/solid';

import { type PartialValues } from '@modular-forms/solid';
import {
  Show,
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { InheritanceLabel, OverrideCheckbox } from '@/entities/settings';
import { cn, useFieldOnChangeSync } from '@/shared/lib';
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
  defaultValues?: Accessor<PartialValues<HooksSettingsSchemaInput> | undefined>;
  onChangePartial?: (values: Partial<HooksSettingsSchemaOutput>) => void;
};

export const HooksSettingsForm: Component<HooksSettingsFormProps> = (props) => {
  const [local, others] = splitProps(props, [
    'overridable',
    'initialValues',
    'defaultValues',
    'onChangePartial',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [form, { Form, Field }] = useHooksSettingsForm();
  useResetHooksSettingsFormValues(form, () => local.initialValues());

  const handleOverrideChange = async (value: boolean) => {
    local.onChangePartial?.({
      overrideHooks: value,
    });
  };

  const updatePreLaunch = useFieldOnChangeSync(
    HooksSettingsSchema,
    form,
    'preLaunch',
    (value) => local.onChangePartial?.({ preLaunch: value }),
  );

  const updateWrapper = useFieldOnChangeSync(
    HooksSettingsSchema,
    form,
    'wrapper',
    (value) => local.onChangePartial?.({ wrapper: value }),
  );

  const updatePostExit = useFieldOnChangeSync(
    HooksSettingsSchema,
    form,
    'postExit',
    (value) => local.onChangePartial?.({ postExit: value }),
  );

  const getPreLaunchDisplayValue = (
    value: Maybe<string>,
    isOverridden: boolean | undefined,
  ) => {
    const defaultPreLaunch =
      local.defaultValues?.()?.preLaunch?.toString() ?? '';

    if (local.overridable && !isOverridden) {
      return defaultPreLaunch;
    }

    return value ?? defaultPreLaunch;
  };

  const getWrapperDisplayValue = (
    value: Maybe<string>,
    isOverridden: boolean | undefined,
  ) => {
    const defaultWrapper = local.defaultValues?.()?.wrapper?.toString() ?? '';

    if (local.overridable && !isOverridden) {
      return defaultWrapper;
    }

    return value ?? defaultWrapper;
  };

  const getPostExitDisplayValue = (
    value: Maybe<string>,
    isOverridden: boolean | undefined,
  ) => {
    const defaultPostExit = local.defaultValues?.()?.postExit?.toString() ?? '';

    if (local.overridable && !isOverridden) {
      return defaultPostExit;
    }

    return value ?? defaultPostExit;
  };

  return (
    <Form class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <Field name='overrideHooks' type='boolean'>
        {(overrideHooks) => (
          <>
            <InheritanceLabel
              label={t('instanceSettings.hookSettings.title')}
              inheritanceLabel={t('settings.usedFromDefaultSettings')}
              isInheritance={local.overridable && !overrideHooks.value}
            />
            <Show when={local.overridable && overrideHooks.value !== undefined}>
              <OverrideCheckbox
                class='mb-1'
                label={t('instanceSettings.customHooksSettings')}
                checked={overrideHooks.value}
                onOverrideChange={handleOverrideChange}
              />
            </Show>

            <LabeledField label={t('instanceSettings.hookSettings.preLaunch')}>
              <Field name='preLaunch' type='string'>
                {(field, inputProps) => (
                  <CombinedTextField
                    disabled={(() =>
                      local.overridable && !overrideHooks.value)()}
                    value={getPreLaunchDisplayValue(
                      field.value,
                      overrideHooks.value,
                    )}
                    label={t('instanceSettings.hookSettings.preLaunchLabel')}
                    labelProps={{ variant: 'description' }}
                    inputProps={{
                      ...inputProps,
                      type: 'text',
                      placeholder: t(
                        'instanceSettings.hookSettings.enterCommandPlaceholder',
                      ),
                      onBlur: (e) => {
                        inputProps.onBlur(e);
                        updatePreLaunch();
                      },
                    }}
                  />
                )}
              </Field>
            </LabeledField>

            <LabeledField label={t('instanceSettings.hookSettings.wrapper')}>
              <Field name='wrapper' type='string'>
                {(field, inputProps) => (
                  <CombinedTextField
                    disabled={(() =>
                      local.overridable && !overrideHooks.value)()}
                    value={getWrapperDisplayValue(
                      field.value,
                      overrideHooks.value,
                    )}
                    label={t('instanceSettings.hookSettings.wrapperLabel')}
                    labelProps={{ variant: 'description' }}
                    inputProps={{
                      ...inputProps,
                      type: 'text',
                      placeholder: t(
                        'instanceSettings.hookSettings.enterCommandPlaceholder',
                      ),
                      onBlur: (e) => {
                        inputProps.onBlur(e);
                        updateWrapper();
                      },
                    }}
                  />
                )}
              </Field>
            </LabeledField>

            <LabeledField label={t('instanceSettings.hookSettings.postExit')}>
              <Field name='postExit' type='string'>
                {(field, inputProps) => (
                  <CombinedTextField
                    disabled={(() =>
                      local.overridable && !overrideHooks.value)()}
                    value={getPostExitDisplayValue(
                      field.value,
                      overrideHooks.value,
                    )}
                    label={t('instanceSettings.hookSettings.postExitLabel')}
                    labelProps={{ variant: 'description' }}
                    inputProps={{
                      ...inputProps,
                      type: 'text',
                      placeholder: t(
                        'instanceSettings.hookSettings.enterCommandPlaceholder',
                      ),
                      onBlur: (e) => {
                        inputProps.onBlur(e);
                        updatePostExit();
                      },
                    }}
                  />
                )}
              </Field>
            </LabeledField>
          </>
        )}
      </Field>
    </Form>
  );
};
