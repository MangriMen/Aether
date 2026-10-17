import { For, Show, type Component } from 'solid-js';

import type {
  StorageKindDto,
  StorageUsageDto,
} from '@/platform/bindings/settings';

import { formatBytes, sumBytes } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { cn } from '@/shared/utils';

const KIND_STYLE = {
  minecraft: {
    label: 'settings.storage.disk_kind.minecraft',
    color: 'bg-[oklch(0.72_0.14_165)]',
  },
  java: {
    label: 'settings.storage.disk_kind.java',
    color: 'bg-[oklch(0.78_0.15_70)]',
  },
  instances: {
    label: 'settings.storage.disk_kind.instances',
    color: 'bg-[oklch(0.68_0.15_285)]',
  },
} as const satisfies Record<StorageKindDto, { label: string; color: string }>;

/** What several instances use together, and what is each instance's own. */
const GROUPS = [
  {
    title: 'settings.storage.group.shared',
    description: 'settings.storage.group.sharedDescription',
    kinds: ['minecraft', 'java'],
  },
  {
    title: 'settings.storage.group.own',
    description: 'settings.storage.group.ownDescription',
    kinds: ['instances'],
  },
] as const satisfies {
  title: string;
  description: string;
  kinds: StorageKindDto[];
}[];

export const DiskUsage: Component<{ usage: StorageUsageDto }> = (props) => {
  const [{ t, locale }] = useTranslation();

  const total = () => sumBytes(props.usage.entries);
  const entriesOf = (kinds: readonly StorageKindDto[]) =>
    props.usage.entries.filter((entry) => kinds.includes(entry.kind));

  return (
    <div class='gap-4 bg-card/card p-4 flex flex-col rounded-md border'>
      <div class='gap-4 flex items-baseline justify-between'>
        <span class='text-lg font-medium'>{t('settings.storage.disk')}</span>
        <span class='font-medium tabular-nums'>
          {t('settings.storage.diskTotal', {
            size: formatBytes(total(), locale()),
          })}
        </span>
      </div>

      <div class='h-3 gap-0.5 flex w-full overflow-hidden rounded-full'>
        <For each={props.usage.entries}>
          {(entry) => (
            <Show when={(entry.bytes ?? 0) > 0}>
              <div
                class={cn('h-full min-w-1', KIND_STYLE[entry.kind].color)}
                style={{ 'flex-grow': entry.bytes ?? 0 }}
              />
            </Show>
          )}
        </For>
      </div>

      <div class='gap-4 @sm/settings-pane:grid-cols-2 grid grid-cols-1'>
        <For each={GROUPS}>
          {(group) => (
            <section class='gap-2 flex flex-col'>
              <div>
                <div class='gap-2 font-medium flex justify-between'>
                  <span>{t(group.title)}</span>
                  <span class='tabular-nums'>
                    {formatBytes(sumBytes(entriesOf(group.kinds)), locale())}
                  </span>
                </div>
                <p class='text-sm text-muted-foreground'>
                  {t(group.description)}
                </p>
              </div>
              <ul class='gap-1 text-sm flex flex-col'>
                <For each={entriesOf(group.kinds)}>
                  {(entry) => (
                    <li class='gap-2 flex items-center'>
                      <span
                        class={cn(
                          'size-2.5 shrink-0 rounded-full',
                          KIND_STYLE[entry.kind].color,
                        )}
                      />
                      <span class='text-muted-foreground grow'>
                        {t(KIND_STYLE[entry.kind].label)}
                      </span>
                      <span class='tabular-nums'>
                        {formatBytes(entry.bytes ?? 0, locale())}
                      </span>
                    </li>
                  )}
                </For>
              </ul>
            </section>
          )}
        </For>
      </div>
    </div>
  );
};
