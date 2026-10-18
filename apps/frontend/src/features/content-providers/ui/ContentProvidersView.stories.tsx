import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { createEffect, createSignal } from 'solid-js';
import { fn } from 'storybook/test';

import { ContentProvidersView } from './ContentProvidersView';

const TWO_PLATFORMS = [
  {
    platform: 'modrinth',
    providers: [
      { id: 'modrinth-plugin#modrinth', name: 'Modrinth (modrinth-plugin)' },
      { id: 'mirror-plugin#mirror', name: 'Modrinth mirror (mirror-plugin)' },
    ],
  },
  {
    platform: 'curseforge',
    providers: [
      { id: 'cf-plugin#cf', name: 'CurseForge (cf-plugin)' },
      { id: 'cf-alt#cf', name: 'CurseForge alternative (cf-alt)' },
      { id: 'cf-third#cf', name: 'CurseForge third (cf-third)' },
    ],
  },
];

const meta: Meta<typeof ContentProvidersView> = {
  title: 'content-providers/ContentProvidersView',
  component: ContentProvidersView,
  tags: ['autodocs'],
  args: { onChoose: fn() },
  argTypes: {
    groups: { control: 'object' },
    selected: { control: 'object' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const TwoPlatforms: Story = {
  args: { groups: TWO_PLATFORMS, selected: { curseforge: 'cf-alt#cf' } },
  render: (args) => {
    const [selected, setSelected] = createSignal(args.selected ?? {});
    createEffect(() => setSelected(args.selected ?? {}));

    return (
      <div class='gap-4 flex w-full flex-col'>
        <ContentProvidersView
          groups={args.groups}
          selected={selected()}
          onChoose={(platform, providerId) => {
            setSelected((current) => ({
              ...current,
              [platform]: providerId ?? undefined,
            }));
            args.onChoose?.(platform, providerId);
          }}
        />
      </div>
    );
  },
};

export const RemovedPlugin: Story = {
  args: {
    groups: [{ platform: 'curseforge', providers: [] }],
    selected: { curseforge: 'gone-plugin#cf' },
  },
  render: (args) => (
    <div class='gap-4 flex w-full flex-col'>
      <ContentProvidersView
        groups={args.groups ?? []}
        selected={args.selected}
      />
    </div>
  ),
};

export const NothingToChoose: Story = {
  args: { groups: [] },
  render: (args) => (
    <div class='gap-4 flex w-full flex-col'>
      <ContentProvidersView
        groups={args.groups ?? []}
        selected={args.selected}
      />
    </div>
  ),
};
