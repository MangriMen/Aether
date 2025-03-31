import type { Component, ComponentProps } from 'solid-js';
import { createEffect, createSignal, splitProps } from 'solid-js';

import { cn, stringToNumber } from '@/shared/lib';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

import { MemorySlider } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

export type CustomMemoryProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  systemMaxMemory: number;
  defaultMaxMemory?: number;
  instanceMaxMemory?: number;
  onChange?: (value: number | null) => void;
};

const minMemory = 512;

const CustomMemory: Component<CustomMemoryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'systemMaxMemory',
    'defaultMaxMemory',
    'instanceMaxMemory',
    'onChange',
    'class',
  ]);

  const [{ t }] = useTranslate();

  const [customMemory, setCustomMemory] = createSignal(false);
  const [memory, setMemory] = createSignal(minMemory);
  const [inputMemory, setInputMemory] = createSignal(minMemory.toString());

  const getClampedMemory = (value: number | null) => {
    if (value === null || value < minMemory) {
      return minMemory;
    } else if (value > local.systemMaxMemory) {
      return local.systemMaxMemory;
    }
    return value;
  };

  const setMemoryValue = (
    value: number | null,
    clampCallback: boolean = true,
  ) => {
    const clampedMemory = getClampedMemory(value);
    const defaultMemory = local.defaultMaxMemory ?? minMemory;
    const memory = value ? clampedMemory : defaultMemory;
    setMemory(memory);
    setInputMemory(memory.toString());
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
    setCustomMemory(!!local.instanceMaxMemory);

    const clampedMemory = getClampedMemory(
      local.instanceMaxMemory ?? local.defaultMaxMemory ?? null,
    );
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
          maxValue={local.systemMaxMemory}
          warningValue={local.systemMaxMemory / 2}
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
            onBlur: (e) => setMemoryValue(stringToNumber(e.target.value)),
            onKeyPress: (e) =>
              e.key === 'Enter' &&
              setMemoryValue(
                stringToNumber((e.target as HTMLInputElement).value),
              ),
          }}
        />
      </div>
    </LabeledField>
  );
};

export default CustomMemory;
