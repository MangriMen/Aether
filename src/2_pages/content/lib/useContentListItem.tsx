import type { Accessor } from 'solid-js';

import { ContentType, type ContentItem } from '@/entities/instances';
import { showDialog, closeDialog } from '@/shared/model';
import { InstallContentDialog } from '@/widgets/install-content-dialog';

import { useContentContext } from '../model/contentContext';

export const useContentListItem = (item: Accessor<ContentItem>) => {
  const [
    context,
    { installContent, createIsContentInstalling, createIsContentInstalled },
  ] = useContentContext();

  const contentMetadataId = () =>
    getContentMetadataId(
      item().providerData,
      context.providerDataContentIdField,
    );

  const isInstalling = createIsContentInstalling(() => contentMetadataId());
  const isInstalled = createIsContentInstalled(
    () => contentMetadataId(),
    () => context.instanceId,
  );

  const requestInstall = () => {
    if (context.instanceId) {
      installContent(item());
    } else if (item().contentType === ContentType.Modpack) {
      console.log('install modpack');
    } else {
      showDialog(
        'installContent',
        () => (
          <InstallContentDialog
            providerId={context.providerId}
            providerDataContentIdField={context.providerDataContentIdField}
            installContent={installContent}
            createIsContentInstalled={createIsContentInstalled}
            createIsContentInstalling={createIsContentInstalling}
            item={item()}
            onClose={() => closeDialog('installContent')}
          />
        ),
        {},
      );
    }
  };

  return {
    requestInstall,
    isInstalling,
    isInstalled,
  };
};

const getContentMetadataId = (
  providerData: Record<string, unknown> | undefined,
  providerDataContentIdField: string | undefined,
) => {
  if (!providerDataContentIdField || !providerData) {
    return;
  }

  const metadataId = providerData[providerDataContentIdField];

  return typeof metadataId === 'string' ? metadataId : undefined;
};
