import type { Component, ComponentProps } from 'solid-js';
import { createEffect, createMemo, createSignal, splitProps } from 'solid-js';

import { cn, stringToNumber } from '@/shared/lib';
import { Checkbox, CombinedTextField, LabeledField } from '@/shared/ui';

import { MemorySlider } from '@/entities/instances';

import { useTranslation } from '@/shared/model';

export type CustomMemoryProps = Omit<ComponentProps<'div'>, 'onChange'> & {
  minMemory: number;
  maxMemory: number;
  defaultMaxMemory?: number;
  instanceMaxMemory?: number;
  onChange?: (value: number | null) => void;
};

const CustomMemory: Component<CustomMemoryProps> = (props) => {
  const [local, others] = splitProps(props, [
    'minMemory',
    'maxMemory',
    'defaultMaxMemory',
    'instanceMaxMemory',
    'onChange',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const minMemory = createMemo(() => Math.max(Math.floor(local.minMemory), 0));
  const maxMemory = createMemo(() => Math.floor(local.maxMemory));
  const warningMemory = createMemo(() => Math.floor(maxMemory() / 2));

  const [customMemory, setCustomMemory] = createSignal(false);
  const [memory, setMemory] = createSignal(0);
  const [inputMemory, setInputMemory] = createSignal('0');

  const getClampedMemory = (value: number | null) => {
    if (value === null || value < minMemory()) {
      return minMemory();
    } else if (value > maxMemory()) {
      return maxMemory();
    }
    return value;
  };

  const setMemoryValue = (
    value: number | null,
    clampCallback: boolean = true,
  ) => {
    const clampedMemory = getClampedMemory(value);
    const defaultMemory = local.defaultMaxMemory ?? minMemory();
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
          minValue={minMemory()}
          maxValue={maxMemory()}
          warningValue={warningMemory()}
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
