import type { TFunction } from '@/shared/i18n';
import type { ShowToastParams } from '@/shared/ui-kit';

import { showToast } from '@/shared/ui-kit';
import { logError } from '@/shared/utils';

import { getTranslatedError, isLauncherError } from './launcherError';

export interface ShowErrorParams extends Omit<ShowToastParams, 'description'> {
  err: Error;
  t: TFunction;
}

export const showError = ({
  err,
  t,
  variant = 'destructive',
  ...params
}: ShowErrorParams) => {
  const isTranslationError = isLauncherError(err);

  const description = isTranslationError
    ? getTranslatedError(err, t)
    : undefined;

  showToast({
    description,
    variant,
    ...params,
  });

  if (!isTranslationError) {
    logError(err);
  }
};
