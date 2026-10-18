import { QueryClient } from '@tanstack/solid-query';
import { describe, expect, it } from 'vitest';

import { settingsCache } from '../settingsCache';
import { settingsKeys } from '../settingsQueryKeys';

describe('settingsCache.setCacheBusy', () => {
  it('stores the flag from the event', async () => {
    const queryClient = new QueryClient();

    await settingsCache.setCacheBusy(queryClient, true);

    expect(queryClient.getQueryData(settingsKeys.cacheBusy())).toBe(true);
  });

  it('is not overwritten by an answer that was already on its way', async () => {
    const queryClient = new QueryClient();
    const inFlight = queryClient
      .fetchQuery({
        queryKey: settingsKeys.cacheBusy(),
        queryFn: () =>
          new Promise<boolean>((resolve) =>
            setTimeout(() => resolve(false), 20),
          ),
      })
      .catch(() => undefined);

    await settingsCache.setCacheBusy(queryClient, true);
    await inFlight;
    await new Promise((resolve) => setTimeout(resolve, 40));

    expect(queryClient.getQueryData(settingsKeys.cacheBusy())).toBe(true);
  });
});
