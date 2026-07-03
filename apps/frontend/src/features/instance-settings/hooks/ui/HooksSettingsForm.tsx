import { getInput, Form, Field } from '@formisch/solid';
import {
  Show,
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { InheritanceLabel, OverrideCheckbox } from '@/entities/settings';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTextField, LabeledField } from '@/shared/ui';

import { useHooksSettingsForm, useResetHooksSettingsFormValues } from '../lib';
import {
  type HooksSettingsSchemaInput,
  type HooksSettingsSchemaOutput,
} from '../model';

export type HooksSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<Partial<HooksSettingsSchemaInput> | undefined>;
  defaultValues?: Accessor<Partial<HooksSettingsSchemaInput> | undefined>;
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

  const form = useHooksSettingsForm();
  useResetHooksSettingsFormValues(form, () => local.initialValues());

  const handleOverrideChange = (value: boolean) => {
    local.onChangePartial?.({
      overrideHooks: value,
    });
  };

  const getPreLaunchDisplayValue = (
    value: string | undefined,
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
    value: string | undefined,
    isOverridden: boolean | undefined,
  ) => {
    const defaultWrapper = local.defaultValues?.()?.wrapper?.toString() ?? '';

    if (local.overridable && !isOverridden) {
      return defaultWrapper;
    }

    return value ?? defaultWrapper;
  };

  const getPostExitDisplayValue = (
    value: string | undefined,
    isOverridden: boolean | undefined,
  ) => {
    const defaultPostExit = local.defaultValues?.()?.postExit?.toString() ?? '';

    if (local.overridable && !isOverridden) {
      return defaultPostExit;
    }

    return value ?? defaultPostExit;
  };

  return (
    <Form
      of={form}
      class={cn('gap-2 flex flex-col', local.class)}
      onSubmit={() => {}}
      {...others}
    >
      <Field of={form} path={['overrideHooks']}>
        {(overrideHooks) => (
          <>
            <InheritanceLabel
              label={t('instanceSettings.hookSettings.title')}
              inheritanceLabel={t('settings.usedFromDefaultSettings')}
              isInheritance={local.overridable && !overrideHooks.input}
            />
            <Show when={local.overridable && overrideHooks.input !== undefined}>
              <OverrideCheckbox
                class='mb-1'
                label={t('instanceSettings.customHooksSettings')}
                checked={overrideHooks.input}
                onOverrideChange={handleOverrideChange}
              />
            </Show>

            <LabeledField label={t('instanceSettings.hookSettings.preLaunch')}>
              <Field of={form} path={['preLaunch']}>
                {(field) => (
                  <CombinedTextField
                    disabled={(() =>
                      local.overridable && !overrideHooks.input)()}
                    value={getPreLaunchDisplayValue(
                      field.input as string | undefined,
                      overrideHooks.input as boolean | undefined,
                    )}
                    label={t('instanceSettings.hookSettings.preLaunchLabel')}
                    labelProps={{ variant: 'description' }}
                    inputProps={{
                      ...field.props,
                      type: 'text',
                      placeholder: t(
                        'instanceSettings.hookSettings.enterCommandPlaceholder',
                      ),
                      onBlur: (e) => {
                        field.props.onBlur?.(e);
                        local.onChangePartial?.({
                          preLaunch: getInput(form, { path: ['preLaunch'] }),
                        });
                      },
                    }}
                  />
                )}
              </Field>
            </LabeledField>

            <LabeledField label={t('instanceSettings.hookSettings.wrapper')}>
              <Field of={form} path={['wrapper']}>
                {(field) => (
                  <CombinedTextField
                    disabled={(() =>
                      local.overridable && !overrideHooks.input)()}
                    value={getWrapperDisplayValue(
                      field.input as string | undefined,
                      overrideHooks.input as boolean | undefined,
                    )}
                    label={t('instanceSettings.hookSettings.wrapperLabel')}
                    labelProps={{ variant: 'description' }}
                    inputProps={{
                      ...field.props,
                      type: 'text',
                      placeholder: t(
                        'instanceSettings.hookSettings.enterCommandPlaceholder',
                      ),
                      onBlur: (e) => {
                        field.props.onBlur?.(e);
                        local.onChangePartial?.({
                          wrapper: getInput(form, { path: ['wrapper'] }),
                        });
                      },
                    }}
                  />
                )}
              </Field>
            </LabeledField>

            <LabeledField label={t('instanceSettings.hookSettings.postExit')}>
              <Field of={form} path={['postExit']}>
                {(field) => (
                  <CombinedTextField
                    disabled={(() =>
                      local.overridable && !overrideHooks.input)()}
                    value={getPostExitDisplayValue(
                      field.input as string | undefined,
                      overrideHooks.input as boolean | undefined,
                    )}
                    label={t('instanceSettings.hookSettings.postExitLabel')}
                    labelProps={{ variant: 'description' }}
                    inputProps={{
                      ...field.props,
                      type: 'text',
                      placeholder: t(
                        'instanceSettings.hookSettings.enterCommandPlaceholder',
                      ),
                      onBlur: (e) => {
                        field.props.onBlur?.(e);
                        local.onChangePartial?.({
                          postExit: getInput(form, { path: ['postExit'] }),
                        });
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
