import {
  Component,
  ComponentProps,
  createEffect,
  createSignal,
  splitProps,
} from 'solid-js';

import { cn, stringToNumber } from '@/shared/lib';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

import { InstanceSettingsTabProps } from '@/entities/instance';

import { DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH } from '../model';

export type WindowTabProps = ComponentProps<'div'> &
  InstanceSettingsTabProps & {
    onChangeWidth?: (value: number) => void;
    onChangeHeight?: (value: number) => void;
  };

const resolutionFieldClass = 'w-[16ch]';

const WindowTab: Component<WindowTabProps> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'onChangeWidth',
    'onChangeHeight',
    'class',
  ]);

  const [custom, setCustom] = createSignal(false);

  const [width, setWidth] = createSignal(DEFAULT_WINDOW_WIDTH.toString());
  const [height, setHeight] = createSignal(DEFAULT_WINDOW_HEIGHT.toString());

  const getClamped = (value: number | undefined, min: number) => {
    if (!value || value < 0) {
      return min;
    }

    return value;
  };

  const setWidthValue = (value: number | undefined) => {
    const clamped = getClamped(value, DEFAULT_WINDOW_WIDTH);
    setWidth(clamped.toString());
    local.onChangeWidth?.(clamped);
  };

  const setHeightValue = (value: number | undefined) => {
    const clamped = getClamped(value, DEFAULT_WINDOW_HEIGHT);
    setHeight(clamped.toString());
    local.onChangeHeight?.(clamped);
  };

  createEffect(() => {
    setWidth(
      getClamped(
        local.instance.gameResolution?.[0],
        DEFAULT_WINDOW_WIDTH,
      ).toString(),
    );
    setHeight(
      getClamped(
        local.instance.gameResolution?.[1],
        DEFAULT_WINDOW_HEIGHT,
      ).toString(),
    );
  });

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <Checkbox
        class='text-sm'
        checked={custom()}
        onChange={setCustom}
        label='Custom window settings'
      />
      <LabeledField class='text-lg' label='Resolution'>
        <div class='flex items-center gap-3'>
          <CombinedTextField
            disabled={!custom()}
            class={resolutionFieldClass}
            value={width()}
            onChange={setWidth}
            inputProps={{
              type: 'text',
              onBlur: (value) =>
                setWidthValue(stringToNumber(value.target.value)),
            }}
          />
          <span class='text-muted-foreground'>&times;</span>
          <CombinedTextField
            disabled={!custom()}
            class={resolutionFieldClass}
            value={height()}
            onChange={setHeight}
            inputProps={{
              type: 'text',
              onBlur: (value) =>
                setHeightValue(stringToNumber(value.target.value)),
            }}
          />
        </div>
      </LabeledField>
    </div>
  );
};

export default WindowTab;
