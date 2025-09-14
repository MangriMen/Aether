import type { Component } from 'solid-js';

import { getValue } from '@modular-forms/solid';
import { createEffect, createMemo, createSignal, splitProps } from 'solid-js';

import type { EditInstance } from '@/entities/instances';

import { useEditInstance } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Checkbox } from '@/shared/ui';

import { useResetWindowFormValues, useWindowForm } from '../../lib';
import { type InstanceSettingsTabProps } from '../../model';
import {
  DEFAULT_WINDOW_HEIGHT,
  DEFAULT_WINDOW_WIDTH,
} from '../../model/window';
import { ResolutionField } from './ResolutionField';

export type WindowTabProps = { class?: string } & InstanceSettingsTabProps;

export const WindowTab: Component<WindowTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'settings', 'class']);

  const [{ t }] = useTranslation();

  const defaultResolution = createMemo(() => ({
    height: local.settings?.gameResolution?.[1] ?? DEFAULT_WINDOW_HEIGHT,
    width: local.settings?.gameResolution?.[0] ?? DEFAULT_WINDOW_WIDTH,
  }));

  const [form, { Form }] = useWindowForm();
  useResetWindowFormValues(form, () => local.instance);

  const { mutateAsync: editInstance } = useEditInstance();

  const editInstanceSimple = (edit: EditInstance) =>
    editInstance({ edit, id: local.instance.id });

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
        checked={isCustom()}
        class='text-sm'
        label={t('instanceSettings.customWindowSettings')}
        onChange={handleChangeIsCustom}
      />
      <ResolutionField
        defaultHeight={defaultResolution().height}
        defaultWidth={defaultResolution().width}
        disabled={!isCustom()}
        form={form}
        onSubmit={handleResolutionSubmit}
      />
    </Form>
  );
};
