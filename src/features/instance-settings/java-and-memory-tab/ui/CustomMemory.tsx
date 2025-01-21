import {
  Component,
  ComponentProps,
  createEffect,
  createSignal,
  splitProps,
} from 'solid-js';

import { cn, stringToNumber } from '@/shared/lib';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

import { MemorySlider } from '@/entities/instance';

export type CustomMemoryProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  defaultValue?: number;
  onChange?: (value: number) => void;
};

const minMemory = 512;
const maxMemory = 32692;

const getClampedMemory = (value: number | undefined) => {
  if (value === undefined || value < minMemory) {
    return minMemory;
  } else if (value > maxMemory) {
    return maxMemory;
  }
  return value;
};

const CustomMemory: Component<CustomMemoryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'defaultValue',
    'onChange',
    'class',
  ]);

  const [customMemory, setCustomMemory] = createSignal(false);
  const [memory, setMemory] = createSignal(minMemory);
  const [inputMemory, setInputMemory] = createSignal(minMemory.toString());

  const setMemoryValue = (value: number | undefined) => {
    const clampedMemory = getClampedMemory(value);
    setMemory(clampedMemory);
    setInputMemory(clampedMemory.toString());
    local.onChange?.(clampedMemory);
  };

  createEffect(() => {
    setCustomMemory(!!local.defaultValue);

    const clampedMemory = getClampedMemory(local.defaultValue);
    setInputMemory(clampedMemory.toString());
    setMemory(clampedMemory);
  });

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label='Memory allocation'
      {...others}
    >
      <Checkbox
        checked={customMemory()}
        onChange={setCustomMemory}
        label='Custom memory'
      />
      <div class='flex items-start gap-4'>
        <MemorySlider
          class='mt-4'
          disabled={!customMemory()}
          minValue={minMemory}
          maxValue={maxMemory}
          value={[memory()]}
          onChange={(value) => setMemoryValue(value[0])}
        />
        <CombinedTextField
          class='w-[9ch]'
          disabled={!customMemory()}
          value={inputMemory()}
          onChange={setInputMemory}
          inputProps={{
            type: 'text',
            onBlur: (value) =>
              setMemoryValue(stringToNumber(value.target.value)),
          }}
        />
      </div>
    </LabeledField>
  );
};

export default CustomMemory;
