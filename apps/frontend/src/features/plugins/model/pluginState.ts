import type {
  PluginDtoState,
  PluginStateReasonDto,
} from '@/platform/bindings/plugin';
import type { TFunction } from '@/shared/i18n';

import {
  getReasonDetails,
  getTranslatedLoadFailure,
  getTranslatedManifestError,
} from '@/features/errors';

export type PluginState = PluginDtoState;

export const isPluginDisabled = (state: PluginState) =>
  state === 'NotLoaded' || state === 'Failed' || state === 'Incompatible';

export const isPluginLoading = (state: PluginState) =>
  state === 'Loading' || state === 'Unloading';

export const isPluginEnabled = (state: PluginState) => state === 'Loaded';

/** Why the core refused to run the plugin, in the user's language. */
export const getTranslatedStateReason = (
  reason: PluginStateReasonDto | null,
  t: TFunction,
): string => {
  if (!reason) {
    return t('plugin.stateReasonUnknown') ?? '';
  }

  switch (reason.code) {
    case 'INCOMPATIBLE_API_VERSION':
      return (
        t('plugin.stateReason.INCOMPATIBLE_API_VERSION', reason.payload) ??
        reason.code
      );
    case 'INVALID_MANIFEST':
      return `${t('plugin.stateReason.INVALID_MANIFEST')}: ${getTranslatedManifestError(reason.payload.error, t)}`;
    case 'LOAD_FAILED':
      return `${t('plugin.stateReason.LOAD_FAILED')}: ${getTranslatedLoadFailure(reason.payload.reason, t)}`;
  }
};

/** The raw text of the failure behind a reason, if nobody could categorize it. Only `ErrorDetails` may show it. */
export const getStateReasonDetails = (
  reason: PluginStateReasonDto | null,
): string | undefined =>
  reason?.code === 'LOAD_FAILED'
    ? getReasonDetails(reason.payload.reason)
    : undefined;

/** Short label for a plugin the core refused to run; `undefined` for any other state. */
export const getTranslatedStateLabel = (
  state: PluginState,
  t: TFunction,
): string | undefined => {
  switch (state) {
    case 'Incompatible':
      return t('plugin.stateIncompatible');
    case 'Failed':
      return t('plugin.stateFailed');
    default:
      return undefined;
  }
};
