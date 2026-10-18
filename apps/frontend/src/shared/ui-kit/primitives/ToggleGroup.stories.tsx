import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import IconMdiBold from '~icons/mdi/format-bold';
import IconMdiItalic from '~icons/mdi/format-italic';
import IconMdiUnderline from '~icons/mdi/format-underline';
import { For } from 'solid-js';
import { fn } from 'storybook/test';

import { ToggleGroup, ToggleGroupItem } from './ToggleGroup';

const VARIANTS = ['default', 'outline'] as const;
const SIZES = ['sm', 'default', 'lg'] as const;
import { orientationArgType } from '../model/storyArgs';

const meta: Meta = {
  title: 'ui-kit/ToggleGroup',
  component: ToggleGroup,
  tags: ['autodocs'],
  args: {
    multiple: true,
    variant: 'default',
    size: 'default',
    disabled: false,
    onChange: fn(),
  },
  argTypes: {
    multiple: { control: 'boolean' },
    variant: { control: 'select', options: VARIANTS },
    size: { control: 'select', options: SIZES },
    disabled: { control: 'boolean' },
    orientation: orientationArgType,
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

const Items = () => (
  <>
    <ToggleGroupItem value='bold' aria-label='Bold'>
      <IconMdiBold class='size-6' />
    </ToggleGroupItem>
    <ToggleGroupItem value='italic' aria-label='Italic'>
      <IconMdiItalic class='size-6' />
    </ToggleGroupItem>
    <ToggleGroupItem value='underline' aria-label='Underline'>
      <IconMdiUnderline class='size-6' />
    </ToggleGroupItem>
  </>
);

export const Multiple: Story = {
  render: (args) => (
    <ToggleGroup {...args}>
      <Items />
    </ToggleGroup>
  ),
};

export const AllVariants: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div class='gap-4 grid'>
      <For each={VARIANTS}>
        {(variant) => (
          <div class='gap-4 flex items-center'>
            <span class='w-20 font-mono text-xs text-muted-foreground'>
              {variant}
            </span>
            <For each={SIZES}>
              {(size) => (
                <ToggleGroup multiple variant={variant} size={size}>
                  <Items />
                </ToggleGroup>
              )}
            </For>
          </div>
        )}
      </For>
    </div>
  ),
};
