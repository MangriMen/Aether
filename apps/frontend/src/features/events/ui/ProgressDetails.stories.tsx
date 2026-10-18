import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import type { ProgressEvent } from '../model/progressEvent';

import { ProgressDetails } from './ProgressDetails';

const make = (
  id: string,
  parentId: string | null,
  extra: Partial<ProgressEvent> = {},
): ProgressEvent => ({
  event: { type: 'file_download', file_name: id },
  progressBarId: id,
  fraction: 0.5,
  message: '',
  stage: 'downloading',
  status: { type: 'running' },
  parentId,
  mirrorOf: null,
  bytesDone: 5 * 1024 * 1024,
  bytesTotal: 10 * 1024 * 1024,
  speed: 1024 * 1024,
  ...extra,
});

const meta: Meta<typeof ProgressDetails> = {
  title: 'events/ProgressDetails',
  component: ProgressDetails,
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Tree: Story = {
  args: {
    payloads: () => [
      make('pack', null),
      make('java.zip', 'pack'),
      make('client.jar', 'pack', {
        status: { type: 'failed', message: 'Connection lost' },
      }),
      make('mods.zip', 'pack', { mirrorOf: 'java.zip' }),
    ],
  },
};
