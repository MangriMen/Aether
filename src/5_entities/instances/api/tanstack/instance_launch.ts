import { useMutation } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import { launchInstanceRaw, stopInstanceRaw } from '../rawApi';

export const useLaunchInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (id: string) => launchInstanceRaw(id),
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
    mutationFn: (uuid: string) => stopInstanceRaw(uuid),
    onError: (err) => {
      showError({
        title: t('instance.stopError'),
        err,
        t,
      });
    },
  }));
};
