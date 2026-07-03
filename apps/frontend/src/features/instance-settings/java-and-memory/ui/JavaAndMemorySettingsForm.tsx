import type { Accessor, Component, ComponentProps } from 'solid-js';

import { Form } from '@formisch/solid';
import { splitProps } from 'solid-js';

import { cn, noop } from '@/shared/lib';

import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '../model';

import {
  useJavaAndMemorySettingsForm,
  useResetJavaAndMemorySettingsForm,
} from '../lib';
import { EnvVarsFieldSection } from './EnvVarsFieldSection';
import { LaunchArgsFieldSection } from './LaunchArgsFieldSection';
import { MemoryFieldSection } from './MemoryFieldSection';

export type JavaAndMemorySettingsFormProps = Omit<
  ComponentProps<'form'>,
  'onSubmit' | 'children'
> & {
  overridable?: boolean;
  initialValues: Accessor<
    Partial<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  defaultValues?: Accessor<
    Partial<JavaAndMemorySettingsSchemaInput> | undefined
  >;
  onChangePartial: (values: Partial<JavaAndMemorySettingsSchemaOutput>) => void;
};

export const JavaAndMemorySettingsForm: Component<
  JavaAndMemorySettingsFormProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'overridable',
    'initialValues',
    'defaultValues',
    'onChangePartial',
    'class',
  ]);

  const form = useJavaAndMemorySettingsForm();
  useResetJavaAndMemorySettingsForm(form, local.initialValues);

  return (
    <Form
      of={form}
      class={cn('gap-4 flex flex-col', local.class)}
      onSubmit={noop}
      {...others}
    >
      <MemoryFieldSection
        form={form}
        overridable={local.overridable}
        defaultValues={local.defaultValues}
        onChangePartial={local.onChangePartial}
      />
      <LaunchArgsFieldSection
        form={form}
        overridable={local.overridable}
        defaultValues={local.defaultValues}
        onChangePartial={local.onChangePartial}
      />
      <EnvVarsFieldSection
        form={form}
        overridable={local.overridable}
        defaultValues={local.defaultValues}
        onChangePartial={local.onChangePartial}
      />
    </Form>
  );
};
