import type { TFunction } from '@/shared/i18n';
import type { ShowToastParams } from '@/shared/ui-kit';

import { logError } from '@/shared/log';
import { showToast } from '@/shared/ui-kit';

import { findErrorAction } from '../model/errorActions';
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
  const renderDetails = details
    ? () => (
        <ErrorDetails
          details={details}
          defaultOpen={isInternalError(err)}
          t={t}
        />
      )
    : undefined;
  // A way to fix the error that a feature registered, shown above the raw text.
  const fix = isTranslationError ? findErrorAction(err) : undefined;

  showToast({
    description,
    action: fix
      ? () => (
          <div class='gap-2 flex flex-col'>
            {fix()}
            {renderDetails?.()}
          </div>
        )
      : renderDetails,
    variant,
    ...params,
  });

  if (!isTranslationError) {
    logError(err);
  }
};
