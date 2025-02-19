import type { Component, ComponentProps } from 'solid-js';
import { createEffect, createSignal, splitProps } from 'solid-js';

import { cn, stringToNumber } from '@/shared/lib';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

import type { InstanceSettingsTabProps } from '@/entities/instances';
import { editInstance } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

import {
  DEFAULT_WINDOW_HEIGHT,
  DEFAULT_WINDOW_WIDTH,
  RESOLUTION_FIELD_CLASS,
} from '../model/window';
import { getClampedResolution } from '../lib';

export type WindowTabProps = ComponentProps<'div'> & InstanceSettingsTabProps;

export const WindowTab: Component<WindowTabProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const [{ t }] = useTranslate();

  const [custom, setCustom] = createSignal(false);

  const [width, setWidth] = createSignal(DEFAULT_WINDOW_WIDTH.toString());
  const [height, setHeight] = createSignal(DEFAULT_WINDOW_HEIGHT.toString());

  const handleSetCustom = (val: boolean) => {
    setCustom(val);
    if (!val) {
      setWindowResolution(null, null);
    } else {
      setWindowResolution(stringToNumber(width()), stringToNumber(height()));
    }
  };

  const setWindowResolution = (width: number | null, height: number | null) => {
    const gameResolution =
      width === null || height === null
        ? undefined
        : ([width, height] as [number, number]);

    editInstance(local.instance.id, { gameResolution });
  };

  const setWidthValue = (value: number | null) => {
    const clamped = getClampedResolution(value, DEFAULT_WINDOW_WIDTH);
    setWidth(clamped.toString());
    setWindowResolution(clamped, stringToNumber(height()));
  };

  const setHeightValue = (value: number | null) => {
    const clamped = getClampedResolution(value, DEFAULT_WINDOW_HEIGHT);
    setHeight(clamped.toString());
    setWindowResolution(stringToNumber(width()), clamped);
  };

  createEffect(() => {
    setCustom(!!local.instance.gameResolution);
    setWidth(
      getClampedResolution(
        local.instance.gameResolution?.[0] ?? null,
        DEFAULT_WINDOW_WIDTH,
      ).toString(),
    );
    setHeight(
      getClampedResolution(
        local.instance.gameResolution?.[1] ?? null,
        DEFAULT_WINDOW_HEIGHT,
      ).toString(),
    );
  });

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Checkbox
        class='text-sm'
        checked={custom()}
        onChange={handleSetCustom}
        label={t('instanceSettings.customWindowSettings')}
      />
      <LabeledField class='text-lg' label={t('instanceSettings.resolution')}>
        <div class='flex items-center gap-3'>
          <CombinedTextField
            disabled={!custom()}
            class={RESOLUTION_FIELD_CLASS}
            value={width()}
            onChange={setWidth}
            inputProps={{
              type: 'number',
              onBlur: (value) =>
                setWidthValue(stringToNumber(value.target.value)),
              min: 0,
              max: 65535,
            }}
          />
          <span class='text-muted-foreground'>&times;</span>
          <CombinedTextField
            disabled={!custom()}
            class={RESOLUTION_FIELD_CLASS}
            value={height()}
            onChange={setHeight}
            inputProps={{
              type: 'number',
              onBlur: (value) =>
                setHeightValue(stringToNumber(value.target.value)),
              min: 0,
              max: 65535,
            }}
          />
        </div>
      </LabeledField>
    </div>
  );
};
