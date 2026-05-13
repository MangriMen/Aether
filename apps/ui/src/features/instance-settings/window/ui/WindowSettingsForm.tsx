import { setValue, type PartialValues } from '@modular-forms/solid';
import { createMemo, Show, splitProps } from 'solid-js';
import { type Accessor, type Component, type ComponentProps } from 'solid-js';

import { ResolutionPicker } from '@/entities/settings';
import { OverrideCheckbox } from '@/entities/settings';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import {
  useResetWindowSettingsFormValues,
  useWindowSettingsForm,
} from '../lib';
import {
  type WindowSettingsSchemaInput,
  type WindowSettingsSchemaOutput,
} from '../model';

export type WindowSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<PartialValues<WindowSettingsSchemaInput> | undefined>;
  defaultValues?: Accessor<
    PartialValues<WindowSettingsSchemaInput> | undefined
  >;
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

  const [form, { Form, Field }] = useWindowSettingsForm();
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
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Field name='overrideWindowSettings' type='boolean'>
        {(overrideWindowSettings) => {
          const isDisabled = createMemo(
            () => local.overridable && !overrideWindowSettings.value,
          );

          return (
            <>
              <Show
                when={
                  local.overridable &&
                  overrideWindowSettings.value !== undefined
                }
              >
                <OverrideCheckbox
                  class='mb-1'
                  label={t('instanceSettings.customWindowSettings')}
                  checked={overrideWindowSettings.value}
                  onOverrideChange={(value) => {
                    setValue(form, 'overrideWindowSettings', value);
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
