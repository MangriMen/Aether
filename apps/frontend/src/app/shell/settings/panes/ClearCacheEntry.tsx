import { Show, type Component } from 'solid-js';

import type { CacheUsage } from '@/features/settings';
import type { SettingsEntryProps } from '@/shared/settings-layout';

import { useClearCache } from '@/features/settings';
import { formatBytes, sumBytes } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';
import { Button } from '@/shared/ui-kit';

export type ClearCacheEntryProps = SettingsEntryProps & {
  usage?: CacheUsage;
  busy?: boolean;
  isError?: boolean;
};

export const ClearCacheEntry: Component<ClearCacheEntryProps> = (props) => {
  const [{ t, locale }] = useTranslation();

  const clearCache = useClearCache();

  const isBlocked = () => clearCache.isPending || (props.busy ?? true);

  return (
    <SettingsEntry
      title={t('settings.storage.cache')}
      description={
        <>
          <span>{t('settings.storage.cacheDescription')}</span>
          <Show when={props.usage}>
            {(usage) => (
              <span class='font-medium block'>
                {t('settings.storage.size', {
                  size: formatBytes(sumBytes(usage().entries), locale()),
                })}
              </span>
            )}
          </Show>
          <Show when={props.busy}>
            <span class='text-warning block'>{t('settings.storage.busy')}</span>
          </Show>
          <Show when={props.isError}>
            <span role='alert' class='text-destructive block'>
              {t('settings.storage.loadFailed')}
            </span>
          </Show>
        </>
      }
      variant={props.variant}
    >
      <Button
        variant='default'
        disabled={isBlocked()}
        onClick={() => clearCache.mutate()}
      >
        {t('settings.storage.clear')}
      </Button>
    </SettingsEntry>
  );
};
