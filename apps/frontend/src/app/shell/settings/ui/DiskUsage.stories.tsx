import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { DiskUsage } from './DiskUsage';

const MB = 1024 * 1024;

const meta: Meta<typeof DiskUsage> = {
  title: 'shell/DiskUsage',
  component: DiskUsage,
  tags: ['autodocs'],
  argTypes: {
    usage: { control: 'object' },
  },
  render: (args) => (
    <div class='w-full'>
      <DiskUsage {...args} />
    </div>
  ),
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Typical: Story = {
  args: {
    usage: {
      entries: [
        { kind: 'minecraft', bytes: 1369 * MB },
        { kind: 'java', bytes: 550 * MB },
        { kind: 'instances', bytes: 3400 * MB },
      ],
    },
  },
};

export const NoInstancesYet: Story = {
  args: {
    usage: {
      entries: [
        { kind: 'minecraft', bytes: 1369 * MB },
        { kind: 'java', bytes: 550 * MB },
        { kind: 'instances', bytes: 0 },
      ],
    },
  },
};
