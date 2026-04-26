import { useMutation } from '@tanstack/solid-query';

import { showError } from '../../../../shared/lib';
import { useTranslation } from '../../../../shared/model';
import { commands } from '../../api';

export const useLaunchInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (id: string) => commands.launch(id),
    onError: (err, id) => {
      showError({
        title: t('instance.launchError', { id: id }),
        err,
        t,
      });
    },
  }));
};

export const useStopInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (uuid: string) => commands.stop(uuid),
    onError: (err) => {
      showError({
        title: t('instance.stopError'),
        err,
        t,
      });
    },
  }));
};
