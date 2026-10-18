import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { orientationArgType } from '../model/storyArgs';
import { Separator } from './Separator';

const meta: Meta = {
  title: 'ui-kit/Separator',
  component: Separator,
  tags: ['autodocs'],
  args: { orientation: 'horizontal' },
  argTypes: {
    orientation: orientationArgType,
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => (
    <div class='h-24 w-64 flex items-center justify-center'>
      <Separator {...args} />
    </div>
  ),
};

export const InContent: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div>
      <div class='space-y-1'>
        <h4 class='text-sm font-medium leading-none'>Radix Primitives</h4>
        <p class='text-sm text-muted-foreground'>
          An open-source UI component library.
        </p>
      </div>
      <Separator class='my-4' />
      <div class='h-5 space-x-4 text-sm flex items-center'>
        <div>Blog</div>
        <Separator orientation='vertical' />
        <div>Docs</div>
        <Separator orientation='vertical' />
        <div>Source</div>
      </div>
    </div>
  ),
};
