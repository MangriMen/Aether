import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type { Instance, EditInstance } from '@/features/instance';
import type { EditInstanceIconDto } from '@/platform/bindings/instance';

import { isEditInstanceEmpty } from '@/features/instance';

import type { GeneralSettingsSchemaOutput } from '../model/generalValidation';

import {
  generalSettingsValuesToEditInstance,
  instanceToGeneralSettingsValues,
} from '../model/instanceSettingsConverter';

export interface UseGeneralSettingsHandler {
  instance: Accessor<Instance>;
  editInstance: Accessor<
    (args: { id: string; edit: EditInstance }) => Promise<unknown>
  >;
  editInstanceIcon: Accessor<
    (editInstanceIcon: EditInstanceIconDto) => Promise<unknown>
  >;
}

export const useGeneralSettingsHandler = ({
  instance,
  editInstance,
  editInstanceIcon,
}: UseGeneralSettingsHandler) => {
  const initialValues = createMemo(() =>
    instanceToGeneralSettingsValues(instance()),
  );

  const onChange = (values: Partial<GeneralSettingsSchemaOutput>) => {
    if (values.icon !== undefined) {
      editInstanceIcon()({
        instanceId: instance().id,
        iconPath: values.icon,
      });
    }

    const edit = generalSettingsValuesToEditInstance(values);

    if (isEditInstanceEmpty(edit)) {
      return;
    }

    editInstance()({ id: instance().id, edit });
  };

  return { initialValues, onChange };
};
