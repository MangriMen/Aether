import { Show, type Component } from 'solid-js';

import type { CacheUsage } from '@/features/settings';
import type { SettingsEntryProps } from '@/shared/settings-layout';

import { CONTENT_QUERY_KEYS } from '@/features/content';
import { useClearCache } from '@/features/settings';
import { formatBytes, sumBytes } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';
import { Button } from '@/shared/ui-kit';

export type ClearCacheEntryProps = SettingsEntryProps & {
  usage?: CacheUsage;
  busy?: boolean;
  /** Whether the usage or the busy state is still being asked for. */
  isLoading?: boolean;
  isError?: boolean;
  onRetry?: () => void;
};

export const ClearCacheEntry: Component<ClearCacheEntryProps> = (props) => {
  const [{ t, locale }] = useTranslation();

  // What the content lists showed from the cache is fetched again.
  const clearCache = useClearCache((queryClient) => {
    void queryClient.invalidateQueries({ queryKey: CONTENT_QUERY_KEYS.SELF });
  });

  // Unknown while the answer is on its way. When it could not be had, the core refuses a clear
  // that is not allowed, so a failed question must not block the button for good.
  const isBlocked = () =>
    clearCache.isPending || props.isLoading === true || props.busy === true;

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
            <Show when={props.onRetry}>
              <Button
                variant='secondary'
                size='sm'
                onClick={() => props.onRetry?.()}
              >
                {t('common.retry')}
              </Button>
            </Show>
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
