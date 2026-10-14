import type { Accessor } from 'solid-js';

import {
  useMutation,
  useMutationState,
  useQuery,
  useQueryClient,
} from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { isLauncherError, useTranslation } from '@/shared/model';
import { showToast } from '@/shared/ui';

import type { EditPluginSettingsDto } from '../api';
import type { PluginId } from './pluginManifest';

import { pluginsCommands } from '../api';
import { pluginsCache, pluginsQueries } from './cache';
import { pluginKeys } from './queryKeys';

export const usePlugins = () => useQuery(pluginsQueries.list);

export const usePlugin = (id: Accessor<PluginId>) => {
  const queryClient = useQueryClient();

  return useQuery(() => pluginsQueries.get(queryClient, id()));
};

export const usePluginSettings = (id: Accessor<PluginId>) =>
  useQuery(() => pluginsQueries.settings(id()));

export const useApiVersion = () => useQuery(pluginsQueries.apiVersion);

export const useSyncPlugins = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: pluginsCommands.sync,
    onSuccess: () => pluginsCache.invalidate.all(queryClient),
    onError: (err) => {
      showError({
        title: t('plugins.syncError'),
        err,
        t,
      });
    },
  }));
};

export const isIncompatibleApiVersionError = (err: unknown): boolean =>
  isLauncherError(err) &&
  err.type === 'plugin' &&
  err.payload.code === 'INCOMPATIBLE_API_VERSION';

/**
 * The core refuses to enable a plugin with a reason (hash mismatch, missing host function, id
 * clash...); it is shown here. A caller that offers to force-enable on an incompatible API
 * version says so with `offersForceEnable`, and that refusal is left to its dialog.
 */
export const useEnablePlugin = (options?: { offersForceEnable?: boolean }) => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: pluginsCommands.enable,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
    onError: (err) => {
      if (options?.offersForceEnable && isIncompatibleApiVersionError(err)) {
        return;
      }

      showError({ title: t('plugin.enableError'), err, t });
    },
  }));
};

export const useForceEnablePlugin = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: pluginsCommands.forceEnable,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
    onError: (err) => {
      showError({ title: t('plugin.enableError'), err, t });
    },
  }));
};

export const useDisablePlugin = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: pluginsCommands.disable,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
  }));
};

/**
 * Restarts a running plugin, so that changed folders take effect. A busy plugin refuses
 * (`Busy`) and the refusal is shown, not retried. Attempts share one mutation key so that
 * {@link useLastPluginReload} can show a refusal from another button.
 */
export const useReloadPlugin = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationKey: pluginKeys.reloadAll(),
    mutationFn: (id: PluginId) => pluginsCommands.reload(id),
    onSettled: (_, __, id) => pluginsCache.invalidate.state(queryClient, id),
    onError: (err) => {
      showError({ title: t('plugin.restartFailed'), err, t });
    },
  }));
};

/**
 * The last restart of the plugin, from whichever button asked for it: `undefined` before any.
 * `submittedAt` lets the caller ignore an attempt that predates what it is showing.
 */
export const useLastPluginReload = (id: Accessor<PluginId>) => {
  const states = useMutationState(() => ({
    filters: { mutationKey: pluginKeys.reloadAll() },
    select: (mutation) => ({
      id: mutation.state.variables,
      status: mutation.state.status,
      error: mutation.state.error,
      submittedAt: mutation.state.submittedAt,
    }),
  }));

  return () =>
    states()
      .filter((state) => state.id === id())
      .at(-1);
};

export const useEditPluginSettings = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      id,
      editSettings,
    }: {
      id: string;
      editSettings: EditPluginSettingsDto;
    }) => pluginsCommands.editSettings(id, editSettings),
    // A running plugin may now need a restart, which its state carries.
    onSuccess: (_, variables) =>
      pluginsCache.invalidate.state(queryClient, variables.id),
    onError: (err, variables) => {
      const plugin = pluginsCache.getData.fromList(queryClient, variables.id);

      const name =
        plugin?.manifest.metadata.name ??
        plugin?.manifest.metadata.id ??
        '"unknown"';

      showError({
        title: t('pluginSettings.failedToUpdateSettings', {
          name,
        }),
        err,
        t,
      });
    },
  }));
};

/**
 * Gives a plugin read access to the folder of a file the user named for it. With `dryRun` nothing
 * changes: the answer says whether the plugin lacks the folder (`needed`), so the user is asked
 * only then. The plugin restarts to see the folder, so a busy plugin refuses and the user retries
 * later.
 */
export const useGrantPluginFolderAccess = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      id,
      filePath,
      dryRun,
    }: {
      id: PluginId;
      filePath: string;
      dryRun: boolean;
    }) => pluginsCommands.grantFolderAccess(id, filePath, dryRun),
    onSuccess: (access, variables) => {
      if (access.type !== 'granted') {
        return;
      }

      pluginsCache.invalidate.state(queryClient, variables.id);

      showToast({
        title: t('pluginSettings.folderAccessGranted'),
        description: access.folder,
      });
    },
    onError: (err) => {
      showError({
        title: t('pluginSettings.folderAccessFailed'),
        err,
        t,
      });
    },
  }));
};

/**
 * Lets a plugin reach the host of a URL the user gave it. With `dryRun` nothing changes: the
 * answer says whether the plugin lacks the host (`needed`), so the user is asked only then. No
 * restart: the plugin's requests read its hosts on every call.
 */
export const useGrantPluginHostAccess = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      id,
      url,
      dryRun,
    }: {
      id: PluginId;
      url: string;
      dryRun: boolean;
    }) => pluginsCommands.grantHostAccess(id, url, dryRun),
    onSuccess: (access, variables) => {
      if (access.type !== 'granted') {
        return;
      }

      queryClient.invalidateQueries({
        queryKey: pluginKeys.settings(variables.id),
      });

      showToast({
        title: t('pluginSettings.hostAccessGranted'),
        description: access.host,
      });
    },
    onError: (err) => {
      showError({
        title: t('pluginSettings.hostAccessFailed'),
        err,
        t,
      });
    },
  }));
};

export const useOpenPluginFolder = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: pluginsCommands.openPluginsFolder,
    onError: (err) => {
      showError({
        title: t('plugins.openPluginsFolderError'),
        err,
        t,
      });
    },
  }));
};

export const useImportPlugins = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: pluginsCommands.import,
    onError: (err) => {
      showError({
        title: t('plugins.failedToImport'),
        err,
        t,
      });
    },
  }));
};

export const useRemovePlugin = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: pluginsCommands.remove,
    onSuccess: (_, id) => pluginsCache.invalidate.full(queryClient, id),
    onError: (err, id) => {
      const plugin = pluginsCache.getData.fromList(queryClient, id);

      const name =
        plugin?.manifest.metadata.name ??
        plugin?.manifest.metadata.id ??
        '"unknown"';

      showError({
        title: t('plugins.failedToRemove', {
          name,
        }),
        err,
        t,
      });
    },
  }));
};

// ── GitHub / Remote source hooks ──

/** Read previously fetched updates data from cache (does NOT auto-fetch). */
export const useUpdatesData = (id: Accessor<PluginId>) =>
  useQuery(() => ({
    ...pluginsQueries.updates(id()),
    enabled: false,
  }));

/** Manually check for updates and cache the result. */
export const useCheckPluginUpdates = () => {
  const queryClient = useQueryClient();
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (id: string) => pluginsCommands.checkForUpdates(id),
    onSuccess: (data, id) => {
      queryClient.setQueryData(pluginKeys.updates(id), data);
    },
    onError: (err) => {
      showError({
        title: t('plugins.updateCheckError'),
        err,
        t,
      });
    },
  }));
};

export const usePluginPreview = (url: Accessor<string>) =>
  useQuery(() => pluginsQueries.preview(url()));

export const useUpdatePlugin = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({ id, targetTag }: { id: string; targetTag?: string }) =>
      pluginsCommands.updatePlugin(id, targetTag ?? null),
    onSuccess: async (_, { id }) => {
      await pluginsCache.invalidate.full(queryClient, id);
      await queryClient.fetchQuery(pluginsQueries.updates(id));
    },
    onError: (err, { id }) => {
      const plugin = pluginsCache.getData.fromList(queryClient, id);

      const name =
        plugin?.manifest.metadata.name ??
        plugin?.manifest.metadata.id ??
        '"unknown"';

      showError({
        title: t('plugins.failedToUpdatePlugin', {
          name,
        }),
        err,
        t,
      });
    },
  }));
};

export const useInstallPluginFromProvider = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      identifier,
      downloadUrl,
      tagName,
      version,
    }: {
      identifier: string;
      downloadUrl: string;
      tagName: string;
      version: string;
    }) =>
      pluginsCommands.installPluginFromProvider(
        'git_hub',
        identifier,
        downloadUrl,
        tagName,
        version,
      ),
    onSuccess: () => pluginsCache.invalidate.all(queryClient),
    onError: (err) => {
      showError({
        title: t('plugins.installFromGithubError'),
        err,
        t,
      });
    },
  }));
};
