import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { JavaAndMemorySettingsSchemaOutput } from '@/features/instance-settings';

import {
  type EditInstance,
  isEditInstanceSettingsEmpty,
  type Instance,
} from '@/features/instance';
import {
  defaultInstanceSettingsToJavaAndMemorySettingsValues,
  type DefaultInstanceSettings,
} from '@/features/instance-defaults';

import {
  instanceSettingsToJavaAndMemorySettingsValues,
  javaAndMemorySettingsValuesToEditInstanceSettings,
} from '../model/instanceSettingsConverter';

export interface UseJavaAndMemorySettingsHandler {
  instance: Accessor<Instance>;
  editInstance: Accessor<
    (args: { id: string; edit: EditInstance }) => Promise<unknown>
  >;
  defaultSettings?: Accessor<DefaultInstanceSettings | undefined>;
}

export const useJavaAndMemorySettingsHandler = ({
  instance,
  defaultSettings,
  editInstance,
}: UseJavaAndMemorySettingsHandler) => {
  const initialValues = createMemo(() =>
    instanceSettingsToJavaAndMemorySettingsValues(instance()),
  );

  const defaultValues = createMemo(() =>
    defaultInstanceSettingsToJavaAndMemorySettingsValues(defaultSettings?.()),
  );

  const onChange = (values: Partial<JavaAndMemorySettingsSchemaOutput>) => {
    const edit = javaAndMemorySettingsValuesToEditInstanceSettings(values);

    if (isEditInstanceSettingsEmpty(edit)) {
      return;
    }

    editInstance()({ id: instance().id, edit });
  };

  return { initialValues, defaultValues, onChange };
};
