import { Show, type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { DiskUsage, useCacheUsage, useStorageUsage } from '@/features/settings';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { ClearCacheEntry } from './ClearCacheEntry';

export type StoragePaneProps = SettingsPaneProps;

export const StoragePane: Component<StoragePaneProps> = (props) => {
  const [{ t }] = useTranslation();

  const usage = useCacheUsage();
  const disk = useStorageUsage();

  return (
    <SettingsPane label={t('settings.tab.storage')} {...props}>
      <Show when={disk.data}>{(data) => <DiskUsage usage={data()} />}</Show>
      <Show when={disk.isError}>
        <span role='alert' class='text-destructive'>
          {t('settings.storage.diskLoadFailed')}
        </span>
      </Show>
      <ClearCacheEntry
        variant='card'
        usage={usage.data}
        isError={usage.isError}
      />
    </SettingsPane>
  );
};
