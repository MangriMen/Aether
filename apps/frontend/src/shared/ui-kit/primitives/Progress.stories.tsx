import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { For } from 'solid-js';

import { Progress, ProgressLabel, ProgressValueLabel } from './Progress';

const VARIANTS = ['default', 'error'] as const;

const meta: Meta = {
  title: 'ui-kit/Progress',
  component: Progress,
  tags: ['autodocs'],
  args: {
    value: 30,
    minValue: 0,
    maxValue: 100,
    variant: 'default',
    indeterminate: false,
  },
  argTypes: {
    value: { control: { type: 'range', min: 0, max: 100, step: 1 } },
    minValue: { control: 'number' },
    maxValue: { control: 'number' },
    variant: { control: 'select', options: VARIANTS },
    indeterminate: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => (
    <Progress
      {...args}
      getValueLabel={({ value, max }) => `${value} of ${max} tasks completed`}
      class='w-75 space-y-1'
    >
      <div class='flex justify-between'>
        <ProgressLabel>Processing...</ProgressLabel>
        <ProgressValueLabel />
      </div>
    </Progress>
  ),
};

export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div class='gap-6 flex flex-col'>
      <For each={VARIANTS}>
        {(variant) => (
          <Progress value={60} variant={variant} class='w-75 space-y-1'>
            <ProgressLabel>{variant}</ProgressLabel>
          </Progress>
        )}
      </For>
    </div>
  ),
};
