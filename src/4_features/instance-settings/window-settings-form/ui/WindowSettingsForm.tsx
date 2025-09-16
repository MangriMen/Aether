import { cn } from '@/shared/lib';

import { splitProps } from 'solid-js';
import { type Accessor, type Component, type ComponentProps } from 'solid-js';

import type { PartialValues } from '@modular-forms/solid';
import { ResolutionField } from '@/entities/settings';
import type {
  WindowSchemaValuesInput,
  WindowSchemaValuesOutput,
} from '../model';
import { useResetWindowFormValues, useWindowForm } from '../lib';

export type WindowSettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  initialValues: Accessor<PartialValues<WindowSchemaValuesInput> | undefined>;
  onChangePartial?: (values: Partial<WindowSchemaValuesOutput>) => void;
};

export const WindowSettingsForm: Component<WindowSettingsFormProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'initialValues',
    'onChangePartial',
    'class',
  ]);

  const [form, { Form }] = useWindowForm();
  useResetWindowFormValues(form, local.initialValues);

  const handleResolutionSubmit = (width: number, height: number) => {
    local.onChangePartial?.({
      resolution: {
        width,
        height,
      },
    });
  };

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <ResolutionField
        form={form}
        defaultWidth={local.initialValues()?.resolution?.width}
        defaultHeight={local.initialValues()?.resolution?.height}
        onSubmit={handleResolutionSubmit}
      />
    </Form>
  );
};
