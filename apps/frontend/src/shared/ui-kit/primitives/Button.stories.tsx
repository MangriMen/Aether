import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import IconMdiCheck from '~icons/mdi/check';
import { For, Show } from 'solid-js';

import { Button } from './Button';

const VARIANTS = [
  'default',
  'secondary',
  'outline',
  'ghost',
  'success',
  'destructive',
  'link',
  'ghostWarning',
  'warning',
] as const;
const SIZES = ['sm', 'default', 'lg', 'icon'] as const;

const meta = {
  title: 'ui-kit/Button',
  component: Button,
  tags: ['autodocs'],
  args: {
    children: 'Button',
    variant: 'default',
    size: 'default',
    disabled: false,
    loading: false,
  },
  argTypes: {
    children: { control: 'text' },
    variant: { control: 'select', options: VARIANTS },
    size: { control: 'select', options: SIZES },
    disabled: { control: 'boolean' },
    loading: { control: 'boolean' },
  },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
export const Destructive: Story = { args: { variant: 'destructive' } };
export const Loading: Story = { args: { variant: 'secondary', loading: true } };
export const Disabled: Story = { args: { variant: 'outline', disabled: true } };

export const AllVariants: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div class='gap-8 md:grid-cols-8 grid grid-cols-3'>
      <For each={VARIANTS}>
        {(variant) => (
          <div class='space-y-4'>
            <p class='font-mono text-xs text-muted-foreground'>{variant}</p>
            <div class='gap-2 flex flex-col'>
              <For each={SIZES}>
                {(size) => (
                  <Button variant={variant} size={size}>
                    <Show when={size !== 'icon'} fallback={<IconMdiCheck />}>
                      Button {size}
                    </Show>
                  </Button>
                )}
              </For>
              <Button variant={variant} disabled>
                Disabled
              </Button>
              <Button variant={variant} loading>
                Loading
              </Button>
            </div>
          </div>
        )}
      </For>
    </div>
  ),
};
