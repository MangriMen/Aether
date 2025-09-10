import type { Component } from 'solid-js';
import { createEffect, createMemo, createSignal, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { Checkbox } from '@/shared/ui';

import type { EditInstance } from '@/entities/instances';
import { useEditInstance } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

import {
  DEFAULT_WINDOW_HEIGHT,
  DEFAULT_WINDOW_WIDTH,
} from '../../model/window';
import { type InstanceSettingsTabProps } from '../../model';
import { ResolutionField } from './ResolutionField';
import { getValue } from '@modular-forms/solid';
import { useResetWindowFormValues, useWindowForm } from '../../lib';
import { instanceToWindowSettingsValues } from '../../model/converter';

export type WindowTabProps = { class?: string } & InstanceSettingsTabProps;

export const WindowTab: Component<WindowTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'globalSettings',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const defaultResolution = createMemo(() => ({
    width: local.globalSettings?.gameResolution?.[0] ?? DEFAULT_WINDOW_WIDTH,
    height: local.globalSettings?.gameResolution?.[1] ?? DEFAULT_WINDOW_HEIGHT,
  }));

  const windowSettingsFormValues = createMemo(() =>
    instanceToWindowSettingsValues(local.instance),
  );

  const [form, { Form }] = useWindowForm();
  useResetWindowFormValues(form, () => windowSettingsFormValues());

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
