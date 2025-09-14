import type { TFunction } from '../model';
import type { ShowToastParams } from '../ui';

import { getTranslatedError, isLauncherError } from '../model';
import { showToast } from '../ui';
import { debugError } from './log';

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
    debugError(err);
  }
};
