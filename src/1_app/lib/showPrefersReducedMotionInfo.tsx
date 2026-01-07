import { createRoot } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { Button, closeToast, showToast } from '@/shared/ui';

export const showPrefersReducedMotionInfo = (
  onEnableAnimations: () => void,
) => {
  createRoot((dispose) => {
    const handleEnableAnimations = () => {
      closeToast(number);
      onEnableAnimations();
      dispose();
    };

    const [{ t }] = useTranslation();

    const number = showToast({
      title: t('settings.animationsDisabled'),
      description: (
        <div class='flex flex-col gap-2'>
          <span>{t('settings.animationsDisabledDescription')}</span>
          <Button class='self-end' onClick={handleEnableAnimations}>
            Включить
          </Button>
        </div>
      ),
    });
  });
};
