import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import type { ProgressEvent } from '../model/progressEvent';

import { ProgressPopover } from './ProgressPopover';

const MB = 1024 * 1024;

const make = (
  id: string,
  extra: Partial<ProgressEvent> = {},
): ProgressEvent => ({
  event: { type: 'file_download', file_name: id },
  progressBarId: id,
  fraction: 0.4,
  message: '',
  stage: 'downloading',
  status: { type: 'running' },
  parentId: null,
  mirrorOf: null,
  bytesDone: 40 * MB,
  bytesTotal: 100 * MB,
  speed: 3 * MB,
  ...extra,
});

const meta: Meta<typeof ProgressPopover> = {
  title: 'events/ProgressPopover',
  component: ProgressPopover,
  decorators: [
    (Story) => (
      <div class='w-80 rounded-md border bg-popover px-1 py-3'>
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof meta>;

/** A finished bar next to a running one: the clear button shows up. */
export const WithFinished: Story = {
  args: {
    payloads: () => [
      make('client.jar'),
      make('java.zip', {
        fraction: 1,
        status: { type: 'done' },
        bytesDone: 100 * MB,
        speed: null,
      }),
    ],
  },
};
