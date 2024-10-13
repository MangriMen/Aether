import {
  COLOR_MODE_STORAGE_KEY,
  ConfigColorMode,
  createLocalStorageManager,
  useColorMode,
} from '@kobalte/core';
import { Component, createMemo, createSignal } from 'solid-js';

import { Option } from '@/shared/model';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';

import { SelectColorModeProps } from './types';

const COLOR_MODE_OPTIONS: Option<ConfigColorMode>[] = [
  { name: 'Light', value: 'light' },
  { name: 'Dark', value: 'dark' },
  { name: 'System', value: 'system' },
];

export const SelectColorMode: Component<SelectColorModeProps> = (props) => {
  const storageManager = createLocalStorageManager(COLOR_MODE_STORAGE_KEY);

  const colorModeContext = useColorMode();

  const [currentColorModeConfig, setCurrentColorModeConfig] =
    createSignal<ConfigColorMode>(
      storageManager.get() ?? colorModeContext.colorMode(),
    );
  const currentOption = createMemo(() =>
    COLOR_MODE_OPTIONS.find(
      (option) => option.value === currentColorModeConfig(),
    ),
  );

  const updateCurrentColorModeConfig = () => {
    const colorModeConfig = storageManager.get();

    if (!colorModeConfig) {
      return;
    }

    setCurrentColorModeConfig(colorModeConfig);
  };

  const handleChangeColorMode = (colorMode: Option<ConfigColorMode> | null) => {
    if (colorMode === null) {
      return;
    }

    colorModeContext.setColorMode(colorMode.value);
    updateCurrentColorModeConfig();
  };

  return (
    <Select
      class='w-32'
      options={COLOR_MODE_OPTIONS}
      optionValue='name'
      optionTextValue='name'
      defaultValue={currentOption()}
      value={currentOption()}
      onChange={handleChangeColorMode}
      itemComponent={(props) => (
        <SelectItem item={props.item}>{props.item.rawValue.name}</SelectItem>
      )}
      {...props}
    >
      <SelectTrigger>
        <SelectValue<Option<ConfigColorMode>>>
          {(state) => state.selectedOption().name}
        </SelectValue>
      </SelectTrigger>
      <SelectContent />
    </Select>
  );
};
