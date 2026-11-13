import type { TFunction } from '@/shared/i18n';
import type { ShowToastParams } from '@/shared/ui-kit';

import { logError } from '@/shared/log';
import { showToast } from '@/shared/ui-kit';

import {
  getErrorDetails,
  getTranslatedError,
  isInternalError,
  isLauncherError,
} from '../model/launcherError';
import { ErrorDetails } from './ErrorDetails';

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

  const details = getErrorDetails(err);

  showToast({
    description,
    action: details ? (
      <ErrorDetails details={details} defaultOpen={isInternalError(err)} />
    ) : undefined,
    variant,
    ...params,
  });

  if (!isTranslationError) {
    logError(err);
  }
};
