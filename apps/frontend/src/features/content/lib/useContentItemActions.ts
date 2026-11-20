import { createMemo, createSignal, type Accessor } from 'solid-js';

import { useInstances } from '@/features/instance';
import { showDialog } from '@/shared/dialogs';

import { usePrefetchCompatibility } from '../api/contentQueries';
import { useContentContext } from '../model/contentContext';
import { type ContentItem } from '../model/contentItem';
import { ContentType } from '../model/contentType';
import { InstallContentDialog } from '../ui/InstallContentDialog';

export const useContentItemActions = (
  item: Accessor<ContentItem | undefined>,
) => {
  const [context, { installContent, createIsInstalling, createIsInstalled }] =
    useContentContext();

  const [isPrefetching, setIsPrefetching] = createSignal(false);

  const prefetchCompatibilityQuery = usePrefetchCompatibility();
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

      await prefetchCompatibilityQuery(instanceIds, {
        providerId: context.providerId,
        contentItem: currentItem,
      });
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
      void installContent(currentItem);
    } else {
      await prefetchCompatibility();
      showInstallContentDialog();
    }
  };

  return { isInstalled, isInstalling, isPrefetching, requestInstall };
};
