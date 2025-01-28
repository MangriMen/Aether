import type { Component, ComponentProps } from 'solid-js';
import { createEffect, createSignal, splitProps } from 'solid-js';

import { cn, stringToNumber } from '@/shared/lib';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

import { MemorySlider } from '@/entities/instance';

// eslint-disable-next-line boundaries/element-types
import { useTranslate } from '@/app/model';

export type CustomMemoryProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  defaultValue?: number;
  onChange?: (value: number | null) => void;
};

const minMemory = 512;
const maxMemory = 32692;

const getClampedMemory = (value: number | null) => {
  if (value === null || value < minMemory) {
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

  const [{ t }] = useTranslate();

  const [customMemory, setCustomMemory] = createSignal(false);
  const [memory, setMemory] = createSignal(minMemory);
  const [inputMemory, setInputMemory] = createSignal(minMemory.toString());

  const setMemoryValue = (
    value: number | null,
    clampCallback: boolean = true,
  ) => {
    const clampedMemory = getClampedMemory(value);
    setMemory(clampedMemory);
    setInputMemory(clampedMemory.toString());
    local.onChange?.(clampCallback ? clampedMemory : value);
  };

  const handleSetCustomMemory = (value: boolean) => {
    setCustomMemory(value);
    if (!value) {
      setMemoryValue(null, false);
    } else {
      setMemoryValue(memory());
    }
  };

  createEffect(() => {
    setCustomMemory(!!local.defaultValue);

    const clampedMemory = getClampedMemory(local.defaultValue ?? null);
    setInputMemory(clampedMemory.toString());
    setMemory(clampedMemory);
  });

  return (
    <LabeledField
      class={cn('text-base', local.class)}
      label={t('instanceSettings.memoryAllocation')}
      {...others}
    >
      <Checkbox
        checked={customMemory()}
        onChange={handleSetCustomMemory}
        label={t('instanceSettings.customMemorySettings')}
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
