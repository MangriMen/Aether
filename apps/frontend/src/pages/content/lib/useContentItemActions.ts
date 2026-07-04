import { useQueryClient } from '@tanstack/solid-query';
import { createMemo, createSignal, type Accessor } from 'solid-js';

import {
  CHECK_COMPATIBILITY_QUERY,
  ContentType,
  type ContentItem,
  useInstances,
} from '@/entities/instances';
import { showDialog } from '@/shared/model';
import { InstallContentDialog } from '@/widgets/install-content-dialog';

import { useContentContext } from '../model';

export const useContentItemActions = (
  item: Accessor<ContentItem | undefined>,
) => {
  const [context, { installContent, createIsInstalling, createIsInstalled }] =
    useContentContext();

  const [isPrefetching, setIsPrefetching] = createSignal(false);

  const queryClient = useQueryClient();
  const instances = useInstances();

  const isInstalled = createMemo(() => {
    const isInstalledFn = createIsInstalled(
      () => item()?.id,
      () => context.instanceId,
    );
    return isInstalledFn();
  });

  const isInstalling = createMemo(() => {
    const isInstallingFn = createIsInstalling(() => item()?.id);
    return isInstallingFn();
  });

  const prefetchCompatibility = async () => {
    const currentItem = item();

    if (isPrefetching() || !currentItem) {
      return;
    }

    setIsPrefetching(true);

    try {
      const instanceIds = instances.data?.map((instance) => instance.id) ?? [];

      await queryClient.prefetchQuery(
        CHECK_COMPATIBILITY_QUERY(instanceIds, {
          providerId: context.providerId,
          contentItem: currentItem,
        }),
      );
    } catch {
      /* empty */
    } finally {
      setIsPrefetching(false);
    }
  };

  const showInstallContentDialog = () => {
    const currentItem = item();

    if (!currentItem) {
      return;
    }

    showDialog('installContent', InstallContentDialog, {
      item: currentItem,
      manager: {
        providerId: () => context.providerId,
        installContent,
        createIsInstalled,
        createIsInstalling,
      },
    });
  };

  const requestInstall = async () => {
    const currentItem = item();

    if (isInstalling() || isPrefetching() || !currentItem) {
      return;
    }

    const isSpecificInstancePage = context.instanceId;
    const isModpack = currentItem.contentType === ContentType.Modpack;

    if (isSpecificInstancePage || isModpack) {
      installContent(currentItem);
    } else {
      await prefetchCompatibility();
      showInstallContentDialog();
    }
  };

  return { isInstalled, isInstalling, isPrefetching, requestInstall };
};
