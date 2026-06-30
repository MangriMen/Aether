import { useEnablePlugin, useForceEnablePlugin } from '@/entities/plugins';
import { closeDialog, isLauncherError, showDialog } from '@/shared/model';

import { ForceEnablePluginDialog } from '../ui/ForceEnablePluginDialog';

const FORCE_ENABLE_DIALOG_ID = 'forceEnablePlugin';

const isIncompatibleApiVersionError = (err: unknown): boolean => {
  if (!isLauncherError(err)) {
    return false;
  }

  return (
    err.type === 'plugin' && err.payload.code === 'INCOMPATIBLE_API_VERSION'
  );
};

const showForceEnableDialog = (
  pluginName: string,
  onForceEnabled: () => void,
) => {
  showDialog(FORCE_ENABLE_DIALOG_ID, ForceEnablePluginDialog, {
    pluginName,
    onConfirm: () => {
      closeDialog(FORCE_ENABLE_DIALOG_ID);
      onForceEnabled();
    },
  });
};

const forceEnableAndResolve = (
  forceEnablePlugin: ReturnType<typeof useForceEnablePlugin>,
  pluginId: string,
  resolve: (value: void) => void,
  reject: (err: unknown) => void,
) => {
  forceEnablePlugin
    .mutateAsync(pluginId)
    .then(() => resolve())
    .catch(reject);
};

export const useForceEnablePluginWithDialog = () => {
  const enablePlugin = useEnablePlugin();
  const forceEnablePlugin = useForceEnablePlugin();

  const tryEnable = async (plugin: {
    state: string;
    manifest: { metadata: { id: string; name: string } };
  }) => {
    const id = plugin.manifest.metadata.id;

    try {
      await enablePlugin.mutateAsync(id);
    } catch (err: unknown) {
      if (isIncompatibleApiVersionError(err)) {
        return new Promise<void>((resolve, reject) => {
          showForceEnableDialog(plugin.manifest.metadata.name, () => {
            forceEnableAndResolve(forceEnablePlugin, id, resolve, reject);
          });
        });
      }

      throw err;
    }
  };

  return {
    tryEnable,
    isPending: enablePlugin.isPending || forceEnablePlugin.isPending,
  };
};
