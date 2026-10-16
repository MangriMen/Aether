import { Show, type Component } from 'solid-js';

import type { CacheUsageDto } from '@/entities/settings';
import type { SettingsEntryProps } from '@/shared/ui';

import { useClearCache } from '@/entities/settings';
import { formatBytes } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, SettingsEntry } from '@/shared/ui';

export type ClearCacheEntryProps = SettingsEntryProps & {
  usage?: CacheUsageDto;
  isError?: boolean;
};

export const ClearCacheEntry: Component<ClearCacheEntryProps> = (props) => {
  const [{ t, locale }] = useTranslation();

  const clearCache = useClearCache();

  const isBlocked = () => clearCache.isPending || (props.usage?.busy ?? true);

  return (
    <div class='gap-2 flex flex-col'>
      <SettingsEntry
        title={t('settings.storage.cache')}
        description={
          <>
            <span>{t('settings.storage.cacheDescription')}</span>
            <Show when={props.usage}>
              {(usage) => (
                <span class='font-medium block'>
                  {t('settings.storage.size', {
                    size: formatBytes(
                      usage().entries.reduce(
                        (sum, entry) => sum + (entry.bytes ?? 0),
                        0,
                      ),
                      locale(),
                    ),
                  })}
                </span>
              )}
            </Show>
            <Show when={props.usage?.busy}>
              <span class='text-warning block'>
                {t('settings.storage.busy')}
              </span>
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
    </div>
  );
};
