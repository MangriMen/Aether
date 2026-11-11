import IconMdiCheck from '~icons/mdi/check';
import IconMdiDownload from '~icons/mdi/download';
import IconMdiSwapVertical from '~icons/mdi/swap-vertical';
import { createMemo, type Accessor } from 'solid-js';

import type { ContentType, ContentVersion } from '@/features/content';

import { useContentContext } from '@/features/content';

/**
 * Installing a version into the instance of the page, or swapping the
 * installed one for it. Shared by the rows of the versions table and the page
 * of a version, so they offer the same thing in the same state.
 */
export const useContentVersionInstall = (
  version: Accessor<Pick<ContentVersion, 'id' | 'contentId'> | undefined>,
  contentType: Accessor<ContentType | undefined>,
) => {
  const [
    context,
    { installContent, createInstalledVersion, createIsInstalling },
  ] = useContentContext();

  const install = () => {
    const current = version();
    const type = contentType();

    if (!current || !type) {
      return;
    }

    installContent(
      { id: current.contentId, contentType: type },
      undefined,
      current.id,
    );
  };

  const installedVersionMemo = createMemo(() =>
    createInstalledVersion(
      () => version()?.contentId,
      () => context.instanceId,
    ),
  );
  const installedVersion = () => installedVersionMemo()();

  const isCurrentVersionInstalled = createMemo(
    () =>
      installedVersion() !== undefined && installedVersion() === version()?.id,
  );

  const isSwap = () =>
    Boolean(installedVersion()) && !isCurrentVersionInstalled();

  const icon = createMemo(() => {
    if (isCurrentVersionInstalled()) {
      return IconMdiCheck;
    }

    return isSwap() ? IconMdiSwapVertical : IconMdiDownload;
  });

  const isInstallingMemo = createMemo(() =>
    createIsInstalling(() => version()?.contentId),
  );

  return {
    install,
    icon,
    isCurrentVersionInstalled,
    isSwap,
    isInstalling: () => isInstallingMemo()(),
  };
};
