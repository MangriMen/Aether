import { useQueryClient } from '@tanstack/solid-query';
import { createSignal, type Accessor } from 'solid-js';

import {
  CHECK_COMPATIBILITY_QUERY,
  ContentType,
  useInstances,
  type ContentItem,
} from '@/entities/instances';
import { showDialog, closeDialog } from '@/shared/model';
import { InstallContentDialog } from '@/widgets/install-content-dialog';

import { useContentContext } from '../model/contentContext';

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
    setIsLoading(true);

    try {
      const instanceIds = instances.data?.map((instance) => instance.id) ?? [];

      await queryClient.prefetchQuery(
        CHECK_COMPATIBILITY_QUERY(instanceIds, {
          provider: context.providerId,
          contentItem: item(),
        }),
      );
    } finally {
      setIsLoading(false);
    }
  };

  const showInstallContentDialog = async () => {
    try {
      await prefetchCompatibility();
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
    if (context.instanceId) {
      installContent(item());
    } else if (item().contentType === ContentType.Modpack) {
      installContent(item());
    } else {
      showInstallContentDialog();
    }
  };

  return {
    requestInstall,
    isInstalling,
    isInstalled,
    isLoading,
  };
};
