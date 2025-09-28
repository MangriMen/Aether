import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { EditInstance, Instance } from '@/entities/instances';
import type { DefaultInstanceSettings } from '@/entities/settings';
import type { WindowSettingsSchemaOutput } from '@/features/instance-settings/window-settings-form';

import { isEditInstanceSettingsEmpty } from '@/entities/instances';
import { defaultInstanceSettingsToWindowSettingsValues } from '@/pages/settings';

import {
  instanceSettingsToWindowSettingsValues,
  windowSettingsValuesToEditInstanceSettings,
} from '../model';

export interface UseWindowSettingsHandler {
  instance: Accessor<Instance>;
  editInstance: Accessor<
    (args: { id: string; edit: EditInstance }) => Promise<unknown>
  >;
  defaultSettings?: Accessor<DefaultInstanceSettings | undefined>;
}

export const useWindowSettingsHandler = ({
  instance,
  defaultSettings,
  editInstance,
}: UseWindowSettingsHandler) => {
  const initialValues = createMemo(() =>
    instanceSettingsToWindowSettingsValues(instance()),
  );

  const defaultValues = createMemo(() =>
    defaultInstanceSettingsToWindowSettingsValues(defaultSettings?.()),
  );

  const onChange = (values: Partial<WindowSettingsSchemaOutput>) => {
    const edit = windowSettingsValuesToEditInstanceSettings(values);

    if (isEditInstanceSettingsEmpty(edit)) {
      return;
    }

    editInstance()({ id: instance().id, edit });
  };

  return { initialValues, defaultValues, onChange };
};
