import type { Component } from 'solid-js';

import { getValue } from '@modular-forms/solid';
import { createEffect, createMemo, createSignal, splitProps } from 'solid-js';

import type { EditInstance } from '@/entities/instances';

import { useEditInstance } from '@/entities/instances';
import { DEFAULT_RESOLUTION, ResolutionField } from '@/entities/settings';
import {
  useResetWindowSettingsFormValues,
  useWindowSettingsForm,
} from '@/features/instance-settings/window-settings-form';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Checkbox } from '@/shared/ui';

import type { InstanceSettingsTabProps } from '../model';

import { instanceSettingsToWindowSettingsValues } from '../model';

export type WindowTabOldProps = InstanceSettingsTabProps & { class?: string };

export const WindowTabOld: Component<WindowTabOldProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'defaultSettings',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const defaultResolution = createMemo(() => ({
    width: local.defaultSettings?.gameResolution?.[0] ?? DEFAULT_RESOLUTION[0],
    height: local.defaultSettings?.gameResolution?.[1] ?? DEFAULT_RESOLUTION[1],
  }));

  const windowSettingsFormValues = createMemo(() =>
    instanceSettingsToWindowSettingsValues(local.instance),
  );

  const [form, { Form }] = useWindowSettingsForm();
  useResetWindowSettingsFormValues(form, () => windowSettingsFormValues());

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceSimple = (edit: EditInstance) =>
    editInstance({ id: local.instance.id, edit });

  const [isCustom, setIsCustom] = createSignal(false);

  createEffect(() => {
    if (local.instance.gameResolution) {
      setIsCustom(true);
    }
  });

  const handleChangeIsCustom = (value: boolean) => {
    setIsCustom(value);

    if (value) {
      const width = getValue(form, 'resolution.width');
      const height = getValue(form, 'resolution.height');

      if (width && height) {
        editInstanceSimple({ gameResolution: [Number(width), Number(height)] });
      }
    } else {
      editInstanceSimple({ gameResolution: null });
    }
  };

  const handleResolutionSubmit = (width: number, height: number) =>
    editInstanceSimple({ gameResolution: [width, height] });

  return (
    <Form class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Checkbox
        class='text-sm'
        checked={isCustom()}
        onChange={handleChangeIsCustom}
        label={t('instanceSettings.customWindowSettings')}
      />
      <ResolutionField
        form={form}
        defaultWidth={defaultResolution().width}
        defaultHeight={defaultResolution().height}
        disabled={!isCustom()}
        onSubmit={handleResolutionSubmit}
      />
    </Form>
  );
};
