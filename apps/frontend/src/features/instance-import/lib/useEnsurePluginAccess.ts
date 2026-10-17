import { useQueryClient } from '@tanstack/solid-query';

import type { PluginId } from '@/features/plugins';

import {
  pluginsCache,
  useGrantPluginFolderAccess,
  useGrantPluginHostAccess,
} from '@/features/plugins';
import { closeDialog, showDialog } from '@/shared/dialogs';
import { useTranslation } from '@/shared/i18n';
import { CombinedDialog } from '@/shared/ui-kit';

import { pluginAccessTargetOf } from '../model/accessTarget';

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

  const ensureAccess = async <Access extends { type: string }>(
    id: PluginId,
    grant: (dryRun: boolean) => Promise<Access>,
    subjectOf: (
      access: Extract<Access, { type: 'needed' }>,
    ) => Record<string, string>,
    keyPrefix: 'host' | 'folder',
  ) => {
    const access = await grant(true);

    if (access.type !== 'needed') {
      return true;
    }

    const texts = {
      name: pluginName(id),
      ...subjectOf(access as Extract<Access, { type: 'needed' }>),
    };
    const allowed = await askUser({
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
