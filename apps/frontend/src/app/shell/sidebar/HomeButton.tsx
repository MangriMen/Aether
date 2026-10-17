import type { Component } from 'solid-js';

import { useNavigate } from '@solidjs/router';
import IconMdiHomeVariant from '~icons/mdi/home-variant';

import type { IconButtonProps } from '@/shared/ui-kit';

import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';

import { APP_ROUTES } from '../appRoutes';

type HomeButtonProps = IconButtonProps;

const HomeButton: Component<HomeButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () => navigate(APP_ROUTES.HOME);

  return (
    <CombinedTooltip
      label={t('home.title')}
      placement='right'
      as={IconButton}
      variant='ghost'
      size='lg'
      icon={IconMdiHomeVariant}
      onClick={handleClick}
      {...props}
    />
  );
};

export default HomeButton;
