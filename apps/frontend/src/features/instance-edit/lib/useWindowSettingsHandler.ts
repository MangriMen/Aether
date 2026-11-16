import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { EditInstance, Instance } from '@/features/instance';
import type { WindowSettingsSchemaOutput } from '@/features/instance-settings';

import { isEditInstanceSettingsEmpty } from '@/features/instance';
import {
  defaultInstanceSettingsToWindowSettingsValues,
  type DefaultInstanceSettings,
} from '@/features/instance-defaults';

import {
  instanceSettingsToWindowSettingsValues,
  windowSettingsValuesToEditInstanceSettings,
} from '../model/instanceSettingsConverter';

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

  // The form handles a failed save: it logs it and puts the stored values back.
  const onChange = async (values: Partial<WindowSettingsSchemaOutput>) => {
    const edit = windowSettingsValuesToEditInstanceSettings(instance(), values);

    if (isEditInstanceSettingsEmpty(edit)) {
      return;
    }

    await editInstance()({ id: instance().id, edit });
  };

  return { initialValues, defaultValues, onChange };
};
