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
  speed: 3 * 1024 * 1024,
};

const unknownSize: ProgressEvent = {
  ...base,
  fraction: 0,
  bytesDone: null,
  bytesTotal: null,
  speed: null,
};

const failed: ProgressEvent = {
  ...base,
  status: { type: 'failed', message: 'Connection lost' },
};

const cancelled: ProgressEvent = { ...base, status: { type: 'cancelled' } };

const mirror: ProgressEvent = { ...base, mirrorOf: 'shared-bar' };

const done: ProgressEvent = {
  ...base,
  fraction: 1,
  status: { type: 'done' },
  bytesDone: base.bytesTotal,
  speed: null,
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

export const UnknownSize: Story = { args: { payload: unknownSize } };

export const Failed: Story = { args: { payload: failed } };

export const Cancelled: Story = { args: { payload: cancelled } };

export const Mirror: Story = { args: { payload: mirror } };

export const Done: Story = { args: { payload: done } };

export const SmallWithBytesAndSpeed: Story = {
  args: { payload: base, size: 'sm' },
};

export const SmallUnknownSize: Story = {
  args: { payload: unknownSize, size: 'sm' },
};

export const SmallFailed: Story = { args: { payload: failed, size: 'sm' } };

export const SmallCancelled: Story = {
  args: { payload: cancelled, size: 'sm' },
};

export const SmallMirror: Story = { args: { payload: mirror, size: 'sm' } };

export const SmallDone: Story = { args: { payload: done, size: 'sm' } };
