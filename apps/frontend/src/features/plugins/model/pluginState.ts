import type {
  PluginDtoState,
  PluginStateReasonDto,
} from '@/platform/bindings/plugin';
import type { TFunction } from '@/shared/model';

export type PluginState = PluginDtoState;

export const isPluginDisabled = (state: PluginState) =>
  state === 'NotLoaded' || state === 'Failed' || state === 'Incompatible';

export const isPluginLoading = (state: PluginState) =>
  state === 'Loading' || state === 'Unloading';

export const isPluginEnabled = (state: PluginState) => state === 'Loaded';

/**
 * Why the core refused to run the plugin, in the user's language; the core's
 * technical details stay as they are inside the message.
 */
export const getTranslatedStateReason = (
  reason: PluginStateReasonDto | null,
  t: TFunction,
): string => {
  if (!reason) {
    return t('plugin.stateReasonUnknown') ?? '';
  }

  return t(`plugin.stateReason.${reason.code}`, reason.payload) ?? reason.code;
};

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
