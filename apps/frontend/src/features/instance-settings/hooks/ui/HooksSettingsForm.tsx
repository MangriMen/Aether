import { Form, Field } from '@formisch/solid';
import {
  For,
  Show,
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { InheritanceLabel, OverrideCheckbox } from '@/entities/settings';
import { cn, noop } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { useHooksSettingsForm, useResetHooksSettingsFormValues } from '../lib';
import {
  FIELDS_CONFIG,
  type HooksSettingsSchemaInput,
  type HooksSettingsSchemaOutput,
} from '../model';
import { HookTextField } from './HookTextField';

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
    local.onChangePartial?.({ overrideHooks: value });
  };

  return (
    <Form
      of={form}
      class={cn('gap-2 flex flex-col', local.class)}
      onSubmit={noop}
      {...others}
    >
      <Field of={form} path={['overrideHooks']}>
        {(overrideHooks) => {
          const isOverridden = () => !!overrideHooks.input;
          const isDisabled = () => !!(local.overridable && !isOverridden());

          return (
            <>
              <InheritanceLabel
                label={t('instanceSettings.hookSettings.title')}
                inheritanceLabel={t('settings.usedFromDefaultSettings')}
                isInheritance={local.overridable && !isOverridden()}
              />
              <Show
                when={local.overridable && overrideHooks.input !== undefined}
              >
                <OverrideCheckbox
                  class='mb-1'
                  label={t('instanceSettings.customHooksSettings')}
                  checked={isOverridden()}
                  onOverrideChange={handleOverrideChange}
                />
              </Show>

              <For each={FIELDS_CONFIG}>
                {(fieldConfig) => (
                  <HookTextField
                    fieldConfig={fieldConfig}
                    form={form}
                    isDisabled={isDisabled}
                    overrideHooksInput={overrideHooks.input}
                    onChangePartial={local.onChangePartial}
                    defaultValues={local.defaultValues}
                    overridable={local.overridable}
                  />
                )}
              </For>
            </>
          );
        }}
      </Field>
    </Form>
  );
};
