import type { JSX } from 'solid-js';

import type { LauncherError } from './launcherError';

/**
 * Something the user can do about an error, shown with it: the renderer of a control that fixes
 * what failed, or `undefined` when the error is not one it knows. The renderer is called while
 * the toast renders, so what it creates is owned by the toast.
 */
export type ErrorAction = (
  error: LauncherError,
) => (() => JSX.Element) | undefined;

const errorActions = new Set<ErrorAction>();

/**
 * Offers an action with every error `showError` shows. The feature that can fix an error owns
 * the action, and the errors feature, which every feature imports, never imports it back: the
 * app registers it once. Returns the function that takes the action back.
 */
export const registerErrorAction = (action: ErrorAction): (() => void) => {
  errorActions.add(action);

  return () => {
    errorActions.delete(action);
  };
};

/** The renderer of the first registered action that knows the error. */
export const findErrorAction = (
  error: LauncherError,
): (() => JSX.Element) | undefined => {
  for (const action of errorActions) {
    const render = action(error);

    if (render) {
      return render;
    }
  }

  return undefined;
};
