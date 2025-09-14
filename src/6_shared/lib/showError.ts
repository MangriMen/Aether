import type { TFunction } from '../model';
import { getTranslatedError, isDebug, isLauncherError } from '../model';
import type { ShowToastParams } from '../ui';
import { showToast } from '../ui';

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

  if (!isTranslationError && isDebug()) {
    console.error(err);
  }
};
