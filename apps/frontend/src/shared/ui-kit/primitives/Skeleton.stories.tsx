import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { Skeleton } from './Skeleton';

const meta: Meta = {
  title: 'ui-kit/Skeleton',
  component: Skeleton,
  tags: ['autodocs'],
  args: {
    width: 250,
    height: 16,
    radius: 10,
    circle: false,
    animate: true,
    visible: true,
  },
  argTypes: {
    width: { control: { type: 'number', min: 1 } },
    height: { control: { type: 'number', min: 1 } },
    radius: { control: { type: 'number', min: 0 } },
    circle: { control: 'boolean' },
    animate: { control: 'boolean' },
    visible: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => <Skeleton {...args} />,
};

export const Composition: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div class='space-x-4 flex items-center'>
      <Skeleton height={48} circle animate={false} />
      <div class='space-y-2'>
        <Skeleton height={16} width={250} radius={10} />
        <Skeleton height={16} width={200} radius={10} />
      </div>
    </div>
  ),
};
