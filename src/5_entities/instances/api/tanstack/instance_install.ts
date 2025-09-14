import { useMutation } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import { installInstanceRaw, updateInstanceRaw } from '../rawApi';

export const useInstallInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ force, id }: { force?: boolean; id: string }) =>
      installInstanceRaw(id, force ?? false),
    onError: (err, { id }) => {
      showError({
        err,
        t,
        title: t('instance.removeError', { id }),
      });
    },
  }));
};

export const useUpdateInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: (id: string) => updateInstanceRaw(id),
    onError: (err, id) => {
      showError({
        err,
        t,
        title: t('instance.removeError', { id }),
      });
    },
  }));
};
