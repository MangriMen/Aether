import type { Component } from 'solid-js';

import { useClearCache } from '@/entities/settings';
import { useTranslation } from '@/shared/model';
import { Button, SettingsEntry, type SettingsEntryProps } from '@/shared/ui';

export type ClearCacheEntryProps = SettingsEntryProps;

export const ClearCacheEntry: Component<ClearCacheEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  const clearCache = useClearCache();

  return (
    <SettingsEntry
      title={t('settings.clearCache')}
      description={t('settings.clearCacheDescription')}
      {...props}
    >
      <Button
        variant='default'
        disabled={clearCache.isPending}
        onClick={() => clearCache.mutate()}
      >
        {t('settings.clearCache')}
      </Button>
    </SettingsEntry>
  );
};
