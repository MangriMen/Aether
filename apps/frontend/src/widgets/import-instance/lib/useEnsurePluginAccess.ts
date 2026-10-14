import { useQueryClient } from '@tanstack/solid-query';

import type { PluginId } from '@/entities/plugins';

import {
  pluginsCache,
  useGrantPluginFolderAccess,
  useGrantPluginHostAccess,
} from '@/entities/plugins';
import { closeDialog, showDialog, useTranslation } from '@/shared/model';
import { CombinedDialog } from '@/shared/ui';

import { pluginAccessTargetOf } from './accessTarget';

const CONFIRM_DIALOG_ID = 'confirmPluginAccess';

/** Resolves with the user's answer; closing the dialog any other way is a "no". */
const askNow = (texts: {
  header?: string;
  description?: string;
  allow?: string;
  deny?: string;
}) =>
  new Promise<boolean>((resolve) => {
    let answered = false;

    const answer = (allowed: boolean) => {
      if (answered) {
        return;
      }

      answered = true;
      closeDialog(CONFIRM_DIALOG_ID);
      resolve(allowed);
    };

    showDialog(CONFIRM_DIALOG_ID, CombinedDialog, {
      header: texts.header,
      description: texts.description,
      buttonOkText: texts.allow,
      buttonCancelText: texts.deny,
      onOk: () => answer(true),
      onCancel: () => answer(false),
      onOpenChange: (open: boolean) => {
        if (!open) {
          answer(false);
        }
      },
    });
  });

type ConfirmTexts = Parameters<typeof askNow>[0];

// The dialogs share one id, so a second one would replace the first and leave its promise
// unanswered: they are shown one after another.
let dialogQueue: Promise<unknown> = Promise.resolve();

const askUser = (texts: ConfirmTexts) => {
  const answer = dialogQueue.then(() => askNow(texts));
  dialogQueue = answer;
  return answer;
};

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

  const pluginName = (id: PluginId) =>
    pluginsCache.getData.fromList(queryClient, id)?.manifest.metadata.name ??
    id;

  const ensureHost = async (id: PluginId, url: string) => {
    const access = await grantHostAccess.mutateAsync({ id, url, dryRun: true });

    if (access.type !== 'needed') {
      return true;
    }

    const texts = { name: pluginName(id), host: access.host };
    const allowed = await askUser({
      header: t('pluginSettings.hostAccessConfirmTitle', texts),
      description: t('pluginSettings.hostAccessConfirmDescription', texts),
      allow: t('pluginSettings.accessConfirmAllow'),
      deny: t('pluginSettings.accessConfirmDeny'),
    });

    if (!allowed) {
      return false;
    }

    await grantHostAccess.mutateAsync({ id, url, dryRun: false });
    return true;
  };

  const ensureFolder = async (id: PluginId, filePath: string) => {
    const access = await grantFolderAccess.mutateAsync({
      id,
      filePath,
      dryRun: true,
    });

    if (access.type !== 'needed') {
      return true;
    }

    const texts = { name: pluginName(id), folder: access.folder };
    const allowed = await askUser({
      header: t('pluginSettings.folderAccessConfirmTitle', texts),
      description: t('pluginSettings.folderAccessConfirmDescription', texts),
      allow: t('pluginSettings.accessConfirmAllow'),
      deny: t('pluginSettings.accessConfirmDeny'),
    });

    if (!allowed) {
      return false;
    }

    await grantFolderAccess.mutateAsync({ id, filePath, dryRun: false });
    return true;
  };

  const ensure = async (id: PluginId, value: string): Promise<boolean> => {
    const target = pluginAccessTargetOf(value);

    try {
      switch (target?.kind) {
        case 'host':
          return await ensureHost(id, target.url);
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
