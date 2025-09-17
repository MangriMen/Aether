import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { HooksSettingsSchemaOutput } from '@/features/instance-settings/hooks-settings-form';

import {
  type EditInstance,
  isEditInstanceSettingsEmpty,
  type Instance,
} from '@/entities/instances';

import {
  hooksSettingsValuesToEditInstanceSettings,
  instanceSettingsToHooksSettingsValues,
} from '../model/converter';

export interface UseHooksSettingsHandler {
  instance: Accessor<Instance>;
  editInstance: Accessor<
    (args: { id: string; edit: EditInstance }) => Promise<unknown>
  >;
}

export const useHooksSettingsHandler = ({
  instance,
  editInstance,
}: UseHooksSettingsHandler) => {
  const initialValues = createMemo(() =>
    instanceSettingsToHooksSettingsValues(instance()),
  );

  const onChange = (values: Partial<HooksSettingsSchemaOutput>) => {
    const edit = hooksSettingsValuesToEditInstanceSettings(values);

    if (isEditInstanceSettingsEmpty(edit)) {
      return;
    }

    editInstance()({ id: instance().id, edit });
  };

  return { initialValues, onChange };
};
