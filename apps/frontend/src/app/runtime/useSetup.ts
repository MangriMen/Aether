import { useMaximizeObserver } from '@/platform/window';

import { usePreventRightClick } from './usePreventRightClick';

export const useSetup = () => {
  usePreventRightClick();
  useMaximizeObserver();
};
