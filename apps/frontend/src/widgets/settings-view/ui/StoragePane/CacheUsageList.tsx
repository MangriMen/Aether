import { For, type Component } from 'solid-js';

import type { CacheUsageEntryDto } from '@/entities/settings';

import { formatBytes } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

const KIND_LABELS = {
  metadata: 'settings.storage.kind.metadata',
  icons: 'settings.storage.kind.icons',
  modpack_archives: 'settings.storage.kind.modpack_archives',
} as const satisfies Record<CacheUsageEntryDto['kind'], string>;

export const CacheUsageList: Component<{
  entries: CacheUsageEntryDto[];
}> = (props) => {
  const [{ t }] = useTranslation();

  const total = () =>
    props.entries.reduce((sum, entry) => sum + (entry.bytes ?? 0), 0);

  return (
    <ul class='mt-4 gap-1 text-sm flex flex-col'>
      <For each={props.entries}>
        {(entry) => (
          <li class='gap-4 flex justify-between'>
            <span class='text-muted-foreground'>
              {t(KIND_LABELS[entry.kind])}
            </span>
            <span class='tabular-nums'>{formatBytes(entry.bytes ?? 0)}</span>
          </li>
        )}
      </For>
      <li class='gap-4 pt-1 font-medium flex justify-between border-t'>
        <span>{t('settings.storage.total')}</span>
        <span class='tabular-nums'>{formatBytes(total())}</span>
      </li>
    </ul>
  );
};
