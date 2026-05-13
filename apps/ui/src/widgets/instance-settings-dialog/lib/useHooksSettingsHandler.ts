import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { DefaultInstanceSettings } from '@/entities/settings';
import type { HooksSettingsSchemaOutput } from '@/features/instance-settings/hooks';

import {
  type EditInstance,
  isEditInstanceSettingsEmpty,
  type Instance,
} from '@/entities/instances';
import { defaultInstanceSettingsToHooksSettingsValues } from '@/pages/settings';

import {
  hooksSettingsValuesToEditInstanceSettings,
  instanceSettingsToHooksSettingsValues,
} from '../model';

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
