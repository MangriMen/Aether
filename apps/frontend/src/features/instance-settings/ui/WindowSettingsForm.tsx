import { setInput, Form, Field } from '@formisch/solid';
import { createMemo, Show, splitProps } from 'solid-js';
import { type Accessor, type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { cn, noop } from '@/shared/utils';

import type {
  WindowSettingsSchemaInput,
  WindowSettingsSchemaOutput,
} from '../model/windowSettingsSchema';

import { revertOnFailure } from '../lib/createFieldSync';
import {
  useResetWindowSettingsFormValues,
  useWindowSettingsForm,
} from '../lib/useWindowSettingsForm';
import { OverrideCheckbox } from './OverrideCheckbox';
import { ResolutionPicker } from './ResolutionPicker';

export type WindowSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<Partial<WindowSettingsSchemaInput> | undefined>;
  defaultValues?: Accessor<Partial<WindowSettingsSchemaInput> | undefined>;
  /** A returned promise that rejects puts the stored values back into the fields. */
  onChangePartial?: (
    values: Partial<WindowSettingsSchemaOutput>,
  ) => Promise<unknown> | void;
};

export const WindowSettingsForm: Component<WindowSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'overridable',
    'initialValues',
    'defaultValues',
    'onChangePartial',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const form = useWindowSettingsForm();
  const revert = useResetWindowSettingsFormValues(form, local.initialValues);

  const save = (values: Partial<WindowSettingsSchemaOutput>) =>
    revertOnFailure(local.onChangePartial?.(values), () => revert(values));

  const handleResolutionSubmit = (width: number, height: number) => {
    save({
      resolution: {
        width,
        height,
      },
    });
  };

  const handleOverrideChange = (value: boolean) => {
    save({
      overrideWindowSettings: value,
    });
  };

  return (
    <Form
      of={form}
      class={cn('gap-2 flex flex-col', local.class)}
      onSubmit={noop}
      {...others}
    >
      <Field of={form} path={['overrideWindowSettings']}>
        {(overrideWindowSettings) => {
          const isDisabled = createMemo(
            () => local.overridable && !overrideWindowSettings.input,
          );

          return (
            <>
              <Show
                when={
                  local.overridable &&
                  overrideWindowSettings.input !== undefined
                }
              >
                <OverrideCheckbox
                  class='mb-1'
                  label={t('instanceSettings.customWindowSettings')}
                  checked={overrideWindowSettings.input}
                  onOverrideChange={(value) => {
                    setInput(form, {
                      path: ['overrideWindowSettings'],
                      input: value,
                    });
                    handleOverrideChange(value);
                  }}
                />
              </Show>

              <ResolutionPicker
                form={form}
                disabled={isDisabled()}
                defaultWidth={local.defaultValues?.()?.resolution?.width}
                defaultHeight={local.defaultValues?.()?.resolution?.height}
                forceDefaultValuesOnDisabled
                onSubmit={handleResolutionSubmit}
              />
            </>
          );
        }}
      </Field>
    </Form>
  );
};
