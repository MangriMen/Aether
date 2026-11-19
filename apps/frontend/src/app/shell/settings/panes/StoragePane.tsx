import { Show, type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/settings-layout';

import {
  useCacheBusy,
  useCacheUsage,
  useStorageUsage,
} from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { SettingsPane } from '@/shared/settings-layout';
import { LoadError, Skeleton } from '@/shared/ui-kit';

import { DiskUsage } from '../ui/DiskUsage';
import { ClearCacheEntry } from './ClearCacheEntry';

export type StoragePaneProps = SettingsPaneProps;

export const StoragePane: Component<StoragePaneProps> = (props) => {
  const [{ t }] = useTranslation();

  const usage = useCacheUsage();
  const busy = useCacheBusy();
  const disk = useStorageUsage();

  return (
    <SettingsPane label={t('settings.tab.storage')} {...props}>
      <Show
        when={disk.data}
        fallback={
          <Show
            when={disk.isError}
            fallback={<Skeleton class='h-24 w-full rounded-lg' />}
          >
            <LoadError
              variant='inline'
              message={t('settings.storage.diskLoadFailed')}
              retryLabel={t('common.retry')}
              onRetry={() => void disk.refetch()}
            />
          </Show>
        }
      >
        {(data) => <DiskUsage usage={data()} />}
      </Show>
      <ClearCacheEntry
        variant='card'
        usage={usage.data}
        busy={busy.data}
        isLoading={usage.isPending || busy.isPending}
        isError={usage.isError || busy.isError}
        onRetry={() => {
          void usage.refetch();
          void busy.refetch();
        }}
      />
    </SettingsPane>
  );
};
