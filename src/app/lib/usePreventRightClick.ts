import { createEffect, onCleanup } from 'solid-js';
import { createPreventRightClick } from '../../shared/lib/createPreventRightClick';
import { RIGHT_CLICK_EXCLUDE_TAGS } from '../config';

export const usePreventRightClick = () => {
  const preventRightClick = createPreventRightClick(RIGHT_CLICK_EXCLUDE_TAGS);

  createEffect(() => {
    document.body.addEventListener('contextmenu', preventRightClick);

    onCleanup(() =>
      document.body.removeEventListener('contextmenu', preventRightClick),
    );
  });
};
