import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useAppSettings } from '@/entities/settings';
import { cn } from '@/shared/lib';
import { useThemeContext } from '@/shared/model';
import { CombinedSlider, CombinedTextField } from '@/shared/ui';

export type BackgroundOpacitySliderProps = ComponentProps<'div'>;

const MIN_PERCENT = 0;
const MAX_PERCENT = 100;
const SCALE_FACTOR = 100;

export const BackgroundOpacitySlider: Component<
  BackgroundOpacitySliderProps
> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const appSettings = useAppSettings();
  const [theme, { setTransparency }] = useThemeContext();

  const isDisabled = createMemo(() => !appSettings.data?.transparent);

  const currentTransparency = createMemo(() =>
    Math.round(theme.transparency * SCALE_FACTOR),
  );

  const updateTransparency = (percent: number) => {
    if (Number.isNaN(percent)) {
      return;
    }

    const clamped = Math.max(MIN_PERCENT, Math.min(MAX_PERCENT, percent));
    setTransparency(clamped / SCALE_FACTOR);
  };

  const handleSliderChange = (values: number[]) => {
    updateTransparency(values[0]);
  };

  const handleTextChange = (value: string) => {
    updateTransparency(parseFloat(value));
  };

  return (
    <div class={cn('flex items-center gap-4', local.class)} {...others}>
      <CombinedSlider
        class='w-36'
        minValue={MIN_PERCENT}
        maxValue={SCALE_FACTOR}
        value={[currentTransparency()]}
        onChange={handleSliderChange}
        disabled={isDisabled()}
      />
      <CombinedTextField
        class='w-[8ch]'
        value={currentTransparency().toString()}
        onChange={handleTextChange}
        disabled={isDisabled()}
        inputProps={{ type: 'number', min: MIN_PERCENT, max: MAX_PERCENT }}
      />
    </div>
  );
};
