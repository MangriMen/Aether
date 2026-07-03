import { setInput } from '@formisch/solid';
import { Form, Field } from '@formisch/solid';
import { createMemo, Show, splitProps } from 'solid-js';
import { type Accessor, type Component, type ComponentProps } from 'solid-js';

import { ResolutionPicker } from '@/entities/settings';
import { OverrideCheckbox } from '@/entities/settings';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import type {
  WindowSettingsSchemaInput,
  WindowSettingsSchemaOutput,
} from '../model';

import {
  useResetWindowSettingsFormValues,
  useWindowSettingsForm,
} from '../lib';

export type WindowSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<Partial<WindowSettingsSchemaInput> | undefined>;
  defaultValues?: Accessor<Partial<WindowSettingsSchemaInput> | undefined>;
  onChangePartial?: (values: Partial<WindowSettingsSchemaOutput>) => void;
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
  useResetWindowSettingsFormValues(form, local.initialValues);

  const handleResolutionSubmit = (width: number, height: number) => {
    local.onChangePartial?.({
      resolution: {
        width,
        height,
      },
    });
  };

  const handleOverrideChange = (value: boolean) => {
    local.onChangePartial?.({
      overrideWindowSettings: value,
    });
  };

  return (
    <Form
      of={form}
      class={cn('gap-2 flex flex-col', local.class)}
      onSubmit={() => {}}
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
