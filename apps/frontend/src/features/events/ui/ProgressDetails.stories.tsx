import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import type { ProgressEvent } from '../model/progressEvent';

import { ProgressDetails } from './ProgressDetails';

const MB = 1024 * 1024;

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
  bytesDone: 5 * MB,
  bytesTotal: 10 * MB,
  speed: MB,
  ...extra,
});

const done = (id: string, parentId: string): ProgressEvent =>
  make(id, parentId, {
    fraction: 1,
    status: { type: 'done' },
    bytesDone: 10 * MB,
    speed: null,
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
      make('pack', null, {
        event: {
          type: 'pack_download',
          instance_path: 'create',
          pack_name: 'Create: Above and Beyond',
          icon: null,
          pack_id: null,
          pack_version: null,
        },
        fraction: 0.35,
        bytesDone: null,
        bytesTotal: null,
        speed: null,
      }),
      make('sodium.jar', 'pack', { fraction: 0.2, speed: 3 * MB }),
      make('lithium.jar', 'pack', { fraction: 0.7, speed: 512 * 1024 }),
      make('broken.jar', 'pack', {
        status: { type: 'failed', message: 'Connection lost' },
      }),
      done('create.jar', 'pack'),
      done('jei.jar', 'pack'),
      make('game', 'pack', {
        event: {
          type: 'minecraft_download',
          instance_id: 'create',
          instance_name: 'Create',
        },
        fraction: 0.6,
        bytesDone: null,
        bytesTotal: null,
        speed: null,
      }),
      make('java.zip', 'game', { fraction: 0.9, speed: 2 * MB }),
      make('java-mirror', 'game', { mirrorOf: 'java.zip' }),
      make('client.jar', 'game', { fraction: 0.3 }),
      done('assets.zip', 'game'),
      make('other-root', null, {
        fraction: 1,
        status: { type: 'done' },
        bytesDone: 10 * MB,
        speed: null,
      }),
    ],
  },
};

/** Opens every node, nested ones included, to look at the whole tree.*/
export const Expanded: Story = {
  args: Tree.args,
  play: async ({ canvasElement }) => {
    for (let round = 0; round < 4; round += 1) {
      await new Promise((resolve) => setTimeout(resolve, 150));
      canvasElement
        .querySelectorAll<HTMLElement>('button[aria-expanded="false"]')
        .forEach((button) => button.click());
    }
  },
};
