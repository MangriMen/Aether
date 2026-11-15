import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { isLauncherError, showError } from '@/features/errors';
import { pluginsCommands } from '@/platform/ipc';
import { useTranslation } from '@/shared/i18n';

import type { CommandAnswer } from '../model/commandApproval';

import { byExpiry } from '../model/commandApproval';
import { pluginKeys } from './pluginKeys';

/** The commands plugins wait to run. The event only says that the list changed. */
export const useCommandApprovals = () =>
  useQuery(() => ({
    queryKey: pluginKeys.commandApprovals(),
    queryFn: async () =>
      byExpiry(await pluginsCommands.listPendingCommandApprovals()),
    staleTime: 0,
  }));

export const invalidateCommandApprovals = (
  queryClient: ReturnType<typeof useQueryClient>,
) => queryClient.invalidateQueries({ queryKey: pluginKeys.commandApprovals() });

/** The question closed before the answer arrived: answered elsewhere, timed out, or its plugin stopped. */
const isQuestionGone = (err: unknown): boolean =>
  isLauncherError(err) &&
  err.type === 'plugin' &&
  err.payload.code === 'COMMAND_APPROVAL_NOT_FOUND';

export const useAnswerCommandApproval = () => {
  const [{ t }] = useTranslation();
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: ({
      requestId,
      answer,
    }: {
      requestId: string;
      answer: CommandAnswer;
    }) => pluginsCommands.answerCommandApproval(requestId, answer),
    onSettled: () => invalidateCommandApprovals(queryClient),
    onError: (err) => {
      if (isQuestionGone(err)) {
        return;
      }

      showError({ title: t('plugin.commandApproval.answerError'), err, t });
    },
  }));
};
