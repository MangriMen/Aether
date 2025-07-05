import { useMutation } from '@tanstack/solid-query';
import { launchInstanceRaw, stopInstanceRaw } from '../rawApi';
import { useTranslation } from '@/shared/model';
import { showError } from '@/shared/lib/showError';

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
