import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { batch, createMemo, onCleanup } from 'solid-js';
import { createStore } from 'solid-js/store';

import type { ProgressEvent } from '../model/progressEvent';

import { buildProgressIndex } from '../model/progressTree';
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
    index: () =>
      buildProgressIndex([
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
      ]),
  },
};

export const Empty: Story = {
  args: { index: () => buildProgressIndex([]) },
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

/**
 * An install that goes on: files download one after another, a new one starts as another ends.
 * The bars live in a store, as in the app, so a bar keeps its identity while it moves on.
 * Look for a list that jumps, a bar that goes back or a frame that drops while it runs.
 */
export const Live: Story = {
  render: () => {
    const FILES = 40;
    const [bars, setBars] = createStore<Record<string, ProgressEvent>>({
      pack: make('pack', null, {
        event: {
          type: 'pack_download',
          instance_path: 'p',
          pack_name: 'Live pack',
          icon: null,
          pack_id: null,
          pack_version: null,
        },
        fraction: 0,
        bytesDone: null,
        bytesTotal: null,
        speed: null,
      }),
    });
    const index = createMemo(() => buildProgressIndex(Object.values(bars)));

    let started = 0;
    const files = () =>
      Object.values(bars).filter((bar) => bar.parentId === 'pack');

    const tick = () => {
      batch(() => {
        for (const bar of files()) {
          if (bar.status.type !== 'running') {
            continue;
          }

          const fraction = Math.min(
            1,
            bar.fraction! + 0.04 + (bar.progressBarId.charCodeAt(5) % 5) * 0.02,
          );
          setBars(
            bar.progressBarId,
            fraction >= 1
              ? done(bar.progressBarId, 'pack')
              : { fraction, bytesDone: Math.round(fraction * 10 * MB) },
          );
        }

        const running = files().filter((bar) => bar.status.type === 'running');

        if (running.length < 4 && started < FILES) {
          started += 1;
          setBars(
            `file-${started}.jar`,
            make(`file-${started}.jar`, 'pack', { fraction: 0 }),
          );
        }

        const finished = files().filter(
          (bar) => bar.status.type === 'done',
        ).length;
        setBars('pack', {
          fraction: finished / FILES,
          status: finished === FILES ? { type: 'done' } : { type: 'running' },
        });
      });
    };

    const timer = setInterval(tick, 150);
    onCleanup(() => clearInterval(timer));

    return (
      <div class='w-xl'>
        <ProgressDetails class='max-h-96' index={index} />
      </div>
    );
  },
};
