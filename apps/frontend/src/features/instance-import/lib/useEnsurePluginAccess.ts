import { useQueryClient } from '@tanstack/solid-query';

import type { PluginId } from '@/features/plugins';

import {
  checkPluginLocalHostAccess,
  pluginsCache,
  useGrantPluginFolderAccess,
  useGrantPluginHostAccess,
  useGrantPluginLocalHostAccess,
} from '@/features/plugins';
import { askConfirmation } from '@/shared/dialogs';
import { useTranslation } from '@/shared/i18n';

import {
  namesLocalhostOrAddress,
  pluginAccessTargetOf,
} from '../model/accessTarget';

// One check per (plugin, value) at a time: the pick and the submit of the same value wait for
// the same answer instead of asking twice.
const inFlight = new Map<string, Promise<boolean>>();

/**
 * Makes sure a plugin can use what the user gave it — the host of a URL, the folder of a file —
 * before it is asked to. Nothing new is given without the user's yes: a dry run tells whether
 * anything is missing (and refuses what the grant would refuse), and only then is the user asked.
 *
 * Resolves to `true` when the plugin has the access (or the value needs none), `false` when the
 * user said no or the grant failed; failures are shown by the mutations themselves.
 */
export const useEnsurePluginAccess = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();
  const grantFolderAccess = useGrantPluginFolderAccess();
  const grantHostAccess = useGrantPluginHostAccess();
  const grantLocalHostAccess = useGrantPluginLocalHostAccess();

  const pluginName = (id: PluginId) =>
    pluginsCache.getData.detail(queryClient, id)?.manifest.metadata.name ?? id;

  const ensureAccess = async <Access extends { type: string }>(
    id: PluginId,
    grant: (dryRun: boolean) => Promise<Access>,
    subjectOf: (
      access: Extract<Access, { type: 'needed' }>,
    ) => Record<string, string>,
    keyPrefix: 'host' | 'localHost' | 'folder',
  ) => {
    const access = await grant(true);

    if (access.type !== 'needed') {
      return true;
    }

    const texts = {
      name: pluginName(id),
      ...subjectOf(access as Extract<Access, { type: 'needed' }>),
    };
    const allowed = await askConfirmation({
      header: t(`pluginSettings.${keyPrefix}AccessConfirmTitle`, texts),
      description: t(
        `pluginSettings.${keyPrefix}AccessConfirmDescription`,
        texts,
      ),
      allow: t('pluginSettings.accessConfirmAllow'),
      deny: t('pluginSettings.accessConfirmDeny'),
    });

    if (!allowed) {
      return false;
    }

    await grant(false);
    return true;
  };

  const ensureHost = (id: PluginId, url: string) =>
    ensureAccess(
      id,
      (dryRun) => grantHostAccess.mutateAsync({ id, url, dryRun }),
      (access) => ({ host: access.host }),
      'host',
    );

  /**
   * A URL on `localhost` or a local address literal can never be an allowed host: it is asked
   * for as a local address, behind the warning that says so. `undefined` when the URL is not
   * one (a public literal, a name): the core judges, so a name on the user's network is asked
   * for as an ordinary host and offered as a local address once the plugin's request is refused.
   */
  const ensureLocalHost = async (
    id: PluginId,
    url: string,
  ): Promise<boolean | undefined> => {
    if (!namesLocalhostOrAddress(url)) {
      return undefined;
    }

    const needed = await checkPluginLocalHostAccess(id, url).catch(
      () => undefined,
    );

    if (!needed) {
      return undefined;
    }

    return ensureAccess(
      id,
      (dryRun) =>
        dryRun
          ? Promise.resolve(needed)
          : grantLocalHostAccess.mutateAsync({ id, url, dryRun }),
      (access) => ({ address: access.host }),
      'localHost',
    );
  };

  const ensureFolder = (id: PluginId, filePath: string) =>
    ensureAccess(
      id,
      (dryRun) => grantFolderAccess.mutateAsync({ id, filePath, dryRun }),
      (access) => ({ folder: access.folder }),
      'folder',
    );

  const ensure = async (id: PluginId, value: string): Promise<boolean> => {
    const target = pluginAccessTargetOf(value);

    try {
      switch (target?.kind) {
        case 'host':
          return (
            (await ensureLocalHost(id, target.url)) ??
            (await ensureHost(id, target.url))
          );
        case 'folder':
          return await ensureFolder(id, target.filePath);
        default:
          return true;
      }
    } catch {
      return false;
    }
  };

  return (id: PluginId, value: string): Promise<boolean> => {
    const key = JSON.stringify([id, value]);
    const pending = inFlight.get(key);

    if (pending) {
      return pending;
    }

    const check = ensure(id, value).finally(() => inFlight.delete(key));
    inFlight.set(key, check);
    return check;
  };
};
