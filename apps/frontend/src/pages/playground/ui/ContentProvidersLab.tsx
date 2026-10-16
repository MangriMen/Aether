import { createSignal } from 'solid-js';

import { ContentProvidersView } from '@/widgets/settings-view';

import { ComponentShelf } from './ComponentShelf';

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

const MISSING_PLUGIN = [
  {
    platform: 'curseforge',
    providers: [],
  },
];

export const ContentProvidersLab = () => {
  const [selected, setSelected] = createSignal<Partial<Record<string, string>>>(
    { curseforge: 'cf-alt#cf' },
  );

  const handleSelect = (platform: string, providerId: string | null) => {
    setSelected((current) => ({
      ...current,
      [platform]: providerId ?? undefined,
    }));
  };

  return (
    <>
      <ComponentShelf title='Content providers: two platforms to choose for'>
        <div class='gap-4 flex w-full flex-col'>
          <ContentProvidersView
            groups={TWO_PLATFORMS}
            selected={selected()}
            onChoose={handleSelect}
          />
        </div>
      </ComponentShelf>
      <ComponentShelf title='Content providers: stored choice of a removed plugin'>
        <div class='gap-4 flex w-full flex-col'>
          <ContentProvidersView
            groups={MISSING_PLUGIN}
            selected={{ curseforge: 'gone-plugin#cf' }}
            onChoose={() => undefined}
          />
        </div>
      </ComponentShelf>
      <ComponentShelf title='Content providers: nothing to choose'>
        <div class='gap-4 flex w-full flex-col'>
          <ContentProvidersView groups={[]} />
        </div>
      </ComponentShelf>
    </>
  );
};
