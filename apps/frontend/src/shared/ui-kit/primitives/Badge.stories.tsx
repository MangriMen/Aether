import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { For } from 'solid-js';

import { Badge } from './Badge';

const VARIANTS = [
  'default',
  'secondary',
  'outline',
  'success',
  'warning',
  'error',
] as const;
const SIZES = ['sm', 'default', 'lg'] as const;

const meta = {
  title: 'ui-kit/Badge',
  component: Badge,
  tags: ['autodocs'],
  args: { children: 'Badge', variant: 'default', size: 'default' },
  argTypes: {
    children: { control: 'text' },
    variant: { control: 'select', options: VARIANTS },
    size: { control: 'select', options: SIZES },
    round: { control: 'boolean' },
  },
} satisfies Meta<typeof Badge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Round: Story = { args: { round: true, children: 'Round' } };
export const Variants: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div class='gap-4 grid'>
      <For each={SIZES}>
        {(size) => (
          <div class='gap-2 flex items-center'>
            <span class='w-16 font-mono text-xs text-muted-foreground'>
              {size}
            </span>
            <For each={VARIANTS}>
              {(variant) => (
                <Badge class='capitalize' variant={variant} size={size}>
                  {variant}
                </Badge>
              )}
            </For>
            <Badge size={size} round>
              Round
            </Badge>
          </div>
        )}
      </For>
    </div>
  ),
};
