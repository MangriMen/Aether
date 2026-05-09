import { useNavigate } from '@solidjs/router';
import IconMdiArrowLeft from '~icons/mdi/arrow-left';
import IconMdiArrowRight from '~icons/mdi/arrow-right';
import { type Component, type ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type BackForwardButtonsProps = ComponentProps<'div'>;

export const BackForwardButtons: Component<BackForwardButtonsProps> = (
  props,
) => {
  const navigate = useNavigate();

  const [{ t }] = useTranslation();

  const handleGoBack = () => navigate(-1);
  const handleGoForward = () => navigate(1);

  return (
    <div class='flex gap-2' {...props}>
      <CombinedTooltip
        label={t('navigation.goBack')}
        as={IconButton}
        size='sm'
        variant='ghost'
        icon={IconMdiArrowLeft}
        onClick={handleGoBack}
      />
      <CombinedTooltip
        label={t('navigation.goForward')}
        as={IconButton}
        size='sm'
        variant='ghost'
        icon={IconMdiArrowRight}
        onClick={handleGoForward}
      />
    </div>
  );
};
