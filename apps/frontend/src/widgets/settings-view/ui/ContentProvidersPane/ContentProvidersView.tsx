import { For, Show, type Component } from 'solid-js';

import { useTranslation } from '@/shared/model';

import type { PlatformProviders } from '../../lib';

import { PlatformProviderEntry } from './PlatformProviderEntry';

export interface ContentProvidersViewProps {
  /** The platforms to choose a provider for. */
  groups: PlatformProviders[];
  /** The stored choice of provider per platform. */
  selected?: Partial<Record<string, string>>;
  /** Replaces saving the choice, for showing the view without settings. */
  onChoose?: (platform: string, providerId: string | null) => void;
}

export const ContentProvidersView: Component<ContentProvidersViewProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  return (
    <Show
      when={props.groups.length > 0}
      fallback={
        <p class='text-muted-foreground'>
          {t('settings.contentProviders.nothingToChoose')}
        </p>
      }
    >
      <For each={props.groups}>
        {(group) => (
          <PlatformProviderEntry
            variant='card'
            platform={group.platform}
            providers={group.providers}
            selected={props.selected?.[group.platform]}
            onChoose={props.onChoose}
          />
        )}
      </For>
    </Show>
  );
};
