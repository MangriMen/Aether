import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { HooksSettingsSchemaOutput } from '@/features/instance-settings';

import {
  type EditInstance,
  isEditInstanceSettingsEmpty,
  type Instance,
} from '@/features/instance';
import {
  defaultInstanceSettingsToHooksSettingsValues,
  type DefaultInstanceSettings,
} from '@/features/instance-defaults';

import {
  hooksSettingsValuesToEditInstanceSettings,
  instanceSettingsToHooksSettingsValues,
} from '../model/instanceSettingsConverter';

export interface UseHooksSettingsHandler {
  instance: Accessor<Instance>;
  editInstance: Accessor<
    (args: { id: string; edit: EditInstance }) => Promise<unknown>
  >;
  defaultSettings?: Accessor<DefaultInstanceSettings | undefined>;
}

export const useHooksSettingsHandler = ({
  instance,
  defaultSettings,
  editInstance,
}: UseHooksSettingsHandler) => {
  const initialValues = createMemo(() =>
    instanceSettingsToHooksSettingsValues(instance()),
  );

  const defaultValues = createMemo(() =>
    defaultInstanceSettingsToHooksSettingsValues(defaultSettings?.()),
  );

  const onChange = (values: Partial<HooksSettingsSchemaOutput>) => {
    const edit = hooksSettingsValuesToEditInstanceSettings(values);

    if (isEditInstanceSettingsEmpty(edit)) {
      return;
    }

    editInstance()({ id: instance().id, edit });
  };

  return { initialValues, defaultValues, onChange };
};
