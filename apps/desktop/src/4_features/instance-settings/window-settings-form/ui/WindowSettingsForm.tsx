import { getValues, type PartialValues } from '@modular-forms/solid';
import { createMemo, Show, splitProps } from 'solid-js';
import { type Accessor, type Component, type ComponentProps } from 'solid-js';

import { ResolutionField } from '@/entities/settings';
import { OverrideCheckbox } from '@/entities/settings';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import {
  useResetWindowSettingsFormValues,
  useWindowSettingsForm,
} from '../lib';
import {
  WindowSettingsSchema,
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

  const [form, { Form }] = useWindowSettingsForm();
  useResetWindowSettingsFormValues(form, local.initialValues);

  const handleResolutionSubmit = (width: number, height: number) => {
    local.onChangePartial?.({
      resolution: {
        width,
        height,
      },
    });
  };

  const isOverride = createMemo(
    () => local.initialValues()?.resolution !== undefined,
  );

  const handleOverrideChange = async (value: boolean) => {
    if (value) {
      const raw = getValues(form, { shouldValid: true });
      const parsed = WindowSettingsSchema.safeParse(raw);

      if (!parsed.success) {
        return;
      }

      local.onChangePartial?.(parsed.data);
    } else {
      local.onChangePartial?.({ resolution: null });
    }
  };

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Show when={local.overridable}>
        <OverrideCheckbox
          class='mb-1'
          label={t('instanceSettings.customWindowSettings')}
          enabledValue={() => true}
          disabledValue={() => false}
          checked={isOverride()}
          onOverrideChange={handleOverrideChange}
        />
      </Show>
      <ResolutionField
        form={form}
        disabled={!isOverride()}
        defaultWidth={local.defaultValues?.()?.resolution?.width}
        defaultHeight={local.defaultValues?.()?.resolution?.height}
        onSubmit={handleResolutionSubmit}
      />
    </Form>
  );
};
