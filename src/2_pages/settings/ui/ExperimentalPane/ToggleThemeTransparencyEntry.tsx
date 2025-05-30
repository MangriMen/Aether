import { useThemeContext } from '@/shared/model';
import { CombinedSlider, CombinedTextField, SettingsEntry } from '@/shared/ui';
import { createMemo, type Component } from 'solid-js';

export type ToggleThemeTransparencyEntryProps = {
  class?: string;
};

export const ToggleThemeTransparencyEntry: Component<
  ToggleThemeTransparencyEntryProps
> = (props) => {
  const [theme, { setTransparency }] = useThemeContext();

  const currentTransparency = createMemo(() =>
    Math.round(theme.transparency * 100),
  );

  const handleChangeTransparencySlider = (transparency: number[]) => {
    setTransparency(transparency[0] / 100);
  };

  const handleChangeTransparencyTextField = (transparency: string) => {
    const transparencyNumber = Number(transparency);
    if (
      Number.isNaN(transparencyNumber) ||
      transparencyNumber > 100 ||
      transparencyNumber < 0
    ) {
      return false;
    }

    setTransparency(transparencyNumber / 100);
  };

  return (
    <SettingsEntry title={'Theme transparency'} {...props}>
      <div class='flex items-center gap-4'>
        <CombinedSlider
          class='w-36'
          minValue={0}
          maxValue={100}
          defaultValue={[0]}
          value={[currentTransparency()]}
          onChange={handleChangeTransparencySlider}
        />
        <CombinedTextField
          class='w-[6ch]'
          value={currentTransparency().toString()}
          onChange={handleChangeTransparencyTextField}
        />
      </div>
    </SettingsEntry>
  );
};
