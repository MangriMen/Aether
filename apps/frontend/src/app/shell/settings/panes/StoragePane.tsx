import { Show, type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/settings-layout';

import {
  useCacheBusy,
  useCacheUsage,
  useStorageUsage,
} from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { SettingsPane } from '@/shared/settings-layout';

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
      <Show when={disk.data}>{(data) => <DiskUsage usage={data()} />}</Show>
      <Show when={disk.isError}>
        <span role='alert' class='text-destructive'>
          {t('settings.storage.diskLoadFailed')}
        </span>
      </Show>
      <ClearCacheEntry
        variant='card'
        usage={usage.data}
        busy={busy.data}
        isError={usage.isError || busy.isError}
      />
    </SettingsPane>
  );
};
