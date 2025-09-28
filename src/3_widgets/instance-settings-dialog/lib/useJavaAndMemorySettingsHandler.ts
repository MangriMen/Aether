import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { DefaultInstanceSettings } from '@/entities/settings';
import type { JavaAndMemorySettingsSchemaOutput } from '@/features/instance-settings/java-and-memory-settings-form';

import {
  type EditInstance,
  isEditInstanceSettingsEmpty,
  type Instance,
} from '@/entities/instances';
import { defaultInstanceSettingsToJavaAndMemorySettingsValues } from '@/pages/settings';

import {
  instanceSettingsToJavaAndMemorySettingsValues,
  javaAndMemorySettingsValuesToEditInstanceSettings,
} from '../model';

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
