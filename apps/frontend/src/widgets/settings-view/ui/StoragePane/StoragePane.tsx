import type { Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { useCacheUsage } from '@/entities/settings';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { ClearCacheEntry } from './ClearCacheEntry';

export type StoragePaneProps = SettingsPaneProps;

export const StoragePane: Component<StoragePaneProps> = (props) => {
  const [{ t }] = useTranslation();

  const usage = useCacheUsage();

  return (
    <SettingsPane label={t('settings.tab.storage')} {...props}>
      <ClearCacheEntry
        variant='card'
        usage={usage.data}
        isError={usage.isError}
      />
    </SettingsPane>
  );
};
