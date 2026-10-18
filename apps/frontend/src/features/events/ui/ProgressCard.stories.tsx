import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import type { ProgressEvent } from '../model/progressEvent';

import { ProgressCard } from './ProgressCard';

const base: ProgressEvent = {
  event: { type: 'file_download', file_name: 'client.jar' },
  progressBarId: 'bar',
  fraction: 0.4,
  message: '',
  stage: 'downloading',
  status: { type: 'running' },
  parentId: null,
  mirrorOf: null,
  bytesDone: 40 * 1024 * 1024,
  bytesTotal: 100 * 1024 * 1024,
  speed: 5 * 1024 * 1024,
};

const meta: Meta<typeof ProgressCard> = {
  title: 'events/ProgressCard',
  component: ProgressCard,
  decorators: [
    (Story) => (
      <div class='w-96'>
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof meta>;

export const WithBytesAndSpeed: Story = { args: { payload: base } };

export const UnknownSize: Story = {
  args: {
    payload: {
      ...base,
      fraction: 0,
      bytesDone: null,
      bytesTotal: null,
      speed: null,
    },
  },
};

export const Failed: Story = {
  args: {
    payload: {
      ...base,
      status: { type: 'failed', message: 'Connection lost' },
    },
  },
};

export const Cancelled: Story = {
  args: { payload: { ...base, status: { type: 'cancelled' } } },
};

export const Mirror: Story = {
  args: { payload: { ...base, mirrorOf: 'shared-bar' } },
};
