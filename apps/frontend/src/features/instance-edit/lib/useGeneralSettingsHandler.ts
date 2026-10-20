import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import type {
  Instance,
  EditInstance,
  EditInstanceIcon,
} from '@/features/instance';

import { isEditInstanceEmpty } from '@/features/instance';
import { logError } from '@/shared/log';

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
    (editInstanceIcon: EditInstanceIcon) => Promise<unknown>
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
      }).catch(logError);
    }

    const edit = generalSettingsValuesToEditInstance(values);

    if (isEditInstanceEmpty(edit)) {
      return;
    }

    editInstance()({ id: instance().id, edit }).catch(logError);
  };

  return { initialValues, onChange };
};
