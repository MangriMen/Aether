import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { For } from 'solid-js';

import { ProgressCircle } from './ProgressCircle';

const SIZES = ['xs', 'sm', 'md', 'lg', 'xl'] as const;

const meta = {
  title: 'ui-kit/ProgressCircle',
  component: ProgressCircle,
  tags: ['autodocs'],
  args: { value: 75, size: 'md', showAnimation: true },
  argTypes: {
    value: { control: { type: 'range', min: 0, max: 100, step: 1 } },
    size: { control: 'select', options: SIZES },
    radius: { control: { type: 'number', min: 1 } },
    strokeWidth: { control: { type: 'number', min: 1 } },
    showAnimation: { control: 'boolean' },
  },
} satisfies Meta<typeof ProgressCircle>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Sizes: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div class='gap-8 flex items-end'>
      <For each={SIZES}>
        {(size) => (
          <div class='gap-2 flex flex-col items-center'>
            <ProgressCircle value={args.value} size={size} />
            <span class='font-mono text-xs text-muted-foreground'>{size}</span>
          </div>
        )}
      </For>
    </div>
  ),
};
