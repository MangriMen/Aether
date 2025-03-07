import { createEffect, onCleanup } from 'solid-js';
import { createPreventRightClick } from '../../shared/lib/createPreventRightClick';
import { RIGHT_CLICK_EXCLUDE_TAGS } from '../config';

/**
 * Hook to prevent right click context menu
 *
 * @remarks
 * This hook listens to the contextmenu event on document.body and prevents
 * the default context menu from showing if the event target is not an input
 * field or a select element.
 */
export const usePreventRightClick = () => {
  const preventRightClick = createPreventRightClick(RIGHT_CLICK_EXCLUDE_TAGS);

  createEffect(() => {
    document.body.addEventListener('contextmenu', preventRightClick);

    onCleanup(() =>
      document.body.removeEventListener('contextmenu', preventRightClick),
    );
  });
};
