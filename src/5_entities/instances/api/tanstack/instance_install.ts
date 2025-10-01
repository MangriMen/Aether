import { useMutation } from '@tanstack/solid-query';

import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';

import { installInstanceRaw, updateInstanceRaw } from '../tauriApi';

export const useInstallInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ id, force }: { id: string; force?: boolean }) =>
      installInstanceRaw(id, force ?? false),
    onError: (err, { id }) => {
      showError({
        title: t('instance.removeError', { id }),
        err,
        t,
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
        title: t('instance.removeError', { id }),
        err,
        t,
      });
    },
  }));
};
