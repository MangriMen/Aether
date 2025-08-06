import type { Component } from 'solid-js';
import { createMemo, onCleanup, splitProps } from 'solid-js';

import { cn, debounce } from '@/shared/lib';

import type { EditInstance } from '@/entities/instances';
import { useEditInstance } from '@/entities/instances';

import { useSettings } from '@/entities/settings';
import {
  MEMORY_SLIDER_HANDLE_DEBOUNCE,
  type InstanceSettingsTabProps,
} from '../../model';
import {
  stringToEnvVars,
  stringToExtraLaunchArgs,
  envVarsToString,
  extraLaunchArgsToString,
} from '../../lib';
import { MemoryField } from './MemoryField';
import { useFieldOnChangeWithMapping } from '../../lib/useFieldMapper';
import {
  useJavaAndMemoryForm,
  useResetJavaAndMemoryFormValues,
} from '../../lib/useJavaAndMemoryForm';
import { ExtraLaunchArgsField } from './ExtraLaunchArgsField';
import { CustomEnvVarsField } from './CustomEnvVarsField';
import { setValue } from '@modular-forms/solid';

export type JavaAndMemoryTabProps = {
  class?: string;
} & InstanceSettingsTabProps;

export const JavaAndMemoryTab: Component<JavaAndMemoryTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const settings = useSettings();

  const [form, { Form, Field }] = useJavaAndMemoryForm();

  useResetJavaAndMemoryFormValues(form, () => local.instance);

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceSimple = (edit: EditInstance) =>
    editInstance({ id: local.instance.id, edit });

  // eslint-disable-next-line solid/reactivity
  const editInstanceSimpleDebounced = debounce(
    editInstanceSimple,
    MEMORY_SLIDER_HANDLE_DEBOUNCE,
  );
  onCleanup(() => {
    editInstanceSimpleDebounced.callAndClear();
  });

  const updateMemory = useFieldOnChangeWithMapping(
    form,
    'memory.maximum',
    (value) => ({
      memory: value ? { maximum: value } : null,
    }),
    editInstanceSimpleDebounced,
  );

  const updateExtraLaunchArgs = useFieldOnChangeWithMapping(
    form,
    'extraLaunchArgs',
    (value) => ({
      extraLaunchArgs: value ? stringToExtraLaunchArgs(value) : null,
    }),
    editInstanceSimple,
  );

  const updateEnvVars = useFieldOnChangeWithMapping(
    form,
    'customEnvVars',
    (value) => ({
      customEnvVars: value ? stringToEnvVars(value) : null,
    }),
    editInstanceSimple,
  );

  const defaultExtraLaunchArgs = createMemo(() =>
    extraLaunchArgsToString(local.instance.extraLaunchArgs),
  );
  const defaultCustomEnvVars = createMemo(() =>
    envVarsToString(local.instance.customEnvVars),
  );

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Field name='memory.maximum' type='number'>
        {(field) => (
          <MemoryField
            value={field.value}
            defaultValue={settings.data?.memory.maximum}
            onChange={updateMemory}
          />
        )}
      </Field>
      <Field name='extraLaunchArgs' type='string'>
        {(field) => (
          <ExtraLaunchArgsField
            value={field.value}
            defaultValue={defaultExtraLaunchArgs()}
            onChange={(value) => setValue(form, 'extraLaunchArgs', value)}
            onBlur={updateExtraLaunchArgs}
          />
        )}
      </Field>
      <Field name='customEnvVars' type='string'>
        {(field) => (
          <CustomEnvVarsField
            value={field.value ?? undefined}
            defaultValue={defaultCustomEnvVars()}
            onChange={(value) => setValue(form, 'customEnvVars', value)}
            onBlur={updateEnvVars}
          />
        )}
      </Field>
    </Form>
  );
};
