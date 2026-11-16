import type { Accessor, Component, ComponentProps } from 'solid-js';

import { Form } from '@formisch/solid';
import { splitProps } from 'solid-js';

import { cn, noop } from '@/shared/utils';

import type {
  JavaAndMemorySettingsSchemaInput,
  JavaAndMemorySettingsSchemaOutput,
} from '../model/javaAndMemorySettingsSchema';

import { revertOnFailure } from '../lib/createFieldSync';
import {
  useJavaAndMemorySettingsForm,
  useResetJavaAndMemorySettingsForm,
} from '../lib/useJavaAndMemorySettingsForm';
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
  /** A returned promise that rejects puts the stored values back into the fields. */
  onChangePartial: (
    values: Partial<JavaAndMemorySettingsSchemaOutput>,
  ) => Promise<unknown> | void;
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
  const revert = useResetJavaAndMemorySettingsForm(form, local.initialValues);

  const save = (values: Partial<JavaAndMemorySettingsSchemaOutput>) =>
    revertOnFailure(local.onChangePartial(values), () => revert(values));

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
        onChangePartial={save}
      />
      <LaunchArgsFieldSection
        form={form}
        overridable={local.overridable}
        defaultValues={local.defaultValues}
        onChangePartial={save}
      />
      <EnvVarsFieldSection
        form={form}
        overridable={local.overridable}
        defaultValues={local.defaultValues}
        onChangePartial={save}
      />
    </Form>
  );
};
