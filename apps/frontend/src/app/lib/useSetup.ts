import { useMaximizeObserver } from '@/shared/lib';

import { usePreventRightClick } from './usePreventRightClick';

export const useSetup = () => {
  usePreventRightClick();
  useMaximizeObserver();
};
