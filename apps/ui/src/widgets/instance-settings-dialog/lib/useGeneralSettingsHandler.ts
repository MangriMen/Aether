import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { Instance, EditInstance } from '@/entities/instances';

import { isEditInstanceEmpty } from '@/entities/instances';

import type { GeneralSettingsSchemaOutput } from '../model';

import {
  generalSettingsValuesToEditInstance,
  instanceToGeneralSettingsValues,
} from '../model';

export interface UseGeneralSettingsHandler {
  instance: Accessor<Instance>;
  editInstance: Accessor<
    (args: { id: string; edit: EditInstance }) => Promise<unknown>
  >;
}

export const useGeneralSettingsHandler = ({
  instance,
  editInstance,
}: UseGeneralSettingsHandler) => {
  const initialValues = createMemo(() =>
    instanceToGeneralSettingsValues(instance()),
  );

  const onChange = (values: Partial<GeneralSettingsSchemaOutput>) => {
    const edit = generalSettingsValuesToEditInstance(values);

    if (isEditInstanceEmpty(edit)) {
      return;
    }

    editInstance()({ id: instance().id, edit });
  };

  return { initialValues, onChange };
};
