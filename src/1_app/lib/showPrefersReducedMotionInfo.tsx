import { createRoot } from 'solid-js';

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

    const number = showToast({
      description: (
        <div class='flex flex-col gap-2'>
          <span>
            Вы можете включить их в приложении, но это может повлиять на
            плавность интерфейса и заряд батареи
          </span>
          <Button class='self-end' onClick={handleEnableAnimations}>
            Включить
          </Button>
        </div>
      ),
      title: 'В системе отключены анимации',
    });
  });
};
