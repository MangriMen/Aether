import { Form, Field } from '@formisch/solid';
import {
  For,
  Show,
  splitProps,
  type Accessor,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { cn, noop } from '@/shared/utils';

import { revertOnFailure } from '../lib/createFieldSync';
import {
  useHooksSettingsForm,
  useResetHooksSettingsFormValues,
} from '../lib/useHooksSettingsForm';
import { FIELDS_CONFIG } from '../model/hooksFields';
import {
  type HooksSettingsSchemaInput,
  type HooksSettingsSchemaOutput,
} from '../model/hooksSettingsSchema';
import { HookTextField } from './HookTextField';
import { InheritanceLabel } from './InheritanceLabel';
import { OverrideCheckbox } from './OverrideCheckbox';

const HINT_KEYS = [
  'instanceSettings.hookSettings.hints.syntax',
  'instanceSettings.hookSettings.hints.quotes',
  'instanceSettings.hookSettings.hints.builtins',
  'instanceSettings.hookSettings.hints.wrapper',
  'instanceSettings.hookSettings.hints.postExit',
] as const;

export type HooksSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<Partial<HooksSettingsSchemaInput> | undefined>;
  defaultValues?: Accessor<Partial<HooksSettingsSchemaInput> | undefined>;
  /** A returned promise that rejects puts the stored values back into the fields. */
  onChangePartial?: (
    values: Partial<HooksSettingsSchemaOutput>,
  ) => Promise<unknown> | void;
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
  const revert = useResetHooksSettingsFormValues(form, () =>
    local.initialValues(),
  );

  const save = (values: Partial<HooksSettingsSchemaOutput>) =>
    revertOnFailure(local.onChangePartial?.(values), () => revert(values));

  const handleOverrideChange = (value: boolean) => {
    save({ overrideHooks: value });
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

              <ul class='text-xs text-muted-foreground mb-1 list-disc pl-4'>
                <For each={HINT_KEYS}>{(key) => <li>{t(key)}</li>}</For>
              </ul>

              <For each={FIELDS_CONFIG}>
                {(fieldConfig) => (
                  <HookTextField
                    fieldConfig={fieldConfig}
                    form={form}
                    isDisabled={isDisabled}
                    overrideHooksInput={overrideHooks.input}
                    onChangePartial={save}
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
