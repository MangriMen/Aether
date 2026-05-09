import { useQueryClient } from '@tanstack/solid-query';
import { createMemo, createSignal, type Accessor } from 'solid-js';

import {
  CHECK_COMPATIBILITY_QUERY,
  ContentType,
  useInstances,
  type ContentItem,
} from '@/entities/instances';
import { ROUTES } from '@/shared/config';
import { searchParamsToQueryString } from '@/shared/lib';
import { showDialog, closeDialog } from '@/shared/model';
import { InstallContentDialog } from '@/widgets/install-content-dialog';

import type { ContentPageSearchParams } from '../model';

import { useContentContext } from '../model';
import { encodeContentSearchParams } from './contentSearchParams';

export const useContentListItem = (item: Accessor<ContentItem>) => {
  const [context, { installContent, createIsInstalling, createIsInstalled }] =
    useContentContext();

  const isInstalling = createIsInstalling(() => item().id);
  const isInstalled = createIsInstalled(
    () => item().id,
    () => context.instanceId,
  );
  const [isLoading, setIsLoading] = createSignal(false);

  const queryClient = useQueryClient();
  const instances = useInstances();

  const prefetchCompatibility = async () => {
    if (isLoading()) {
      return;
    }

    setIsLoading(true);

    try {
      const instanceIds = instances.data?.map((instance) => instance.id) ?? [];

      await queryClient.prefetchQuery(
        CHECK_COMPATIBILITY_QUERY(instanceIds, {
          providerId: context.providerId,
          contentItem: item(),
        }),
      );
    } catch {
      /* empty */
    } finally {
      setIsLoading(false);
    }
  };

  const showInstallContentDialog = async () => {
    try {
      await prefetchCompatibility();
    } catch {
      /* empty */
    } finally {
      showDialog(
        'installContent',
        () => (
          <InstallContentDialog
            item={item()}
            manager={{
              providerId: () => context.providerId,
              installContent,
              createIsInstalled,
              createIsInstalling,
            }}
            onClose={() => closeDialog('installContent')}
          />
        ),
        {},
      );
    }
  };

  const requestInstall = async () => {
    if (isInstalling() || isLoading()) {
      return;
    }

    if (context.instanceId) {
      installContent(item());
    } else if (item().contentType === ContentType.Modpack) {
      installContent(item());
    } else {
      showInstallContentDialog();
    }
  };

  const contentPageHref = createMemo(() => {
    const params: ContentPageSearchParams = {
      instanceId: context.instanceId,
      providerId: context.providerId,
    };

    const encodedFilters = encodeContentSearchParams(params);
    const queryString = searchParamsToQueryString(encodedFilters);

    return `${ROUTES.CONTENT_ITEM(item().id)}?${queryString ? queryString : ''}`;
  });

  return {
    requestInstall,
    isInstalling,
    isInstalled,
    isLoading,
    contentPageHref,
  };
};
