import { useMutation } from '@tanstack/solid-query';

import { showError } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { instanceCommands } from '../../api';

export const useInstallInstance = () => {
  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: ({ id, force }: { id: string; force?: boolean }) =>
      instanceCommands.install(id, force ?? false),
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
    mutationFn: (id: string) => instanceCommands.update(id),
    onError: (err, id) => {
      showError({
        title: t('instance.updateError', { id }),
        err,
        t,
      });
    },
  }));
};
