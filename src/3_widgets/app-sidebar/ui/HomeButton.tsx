import type { Component } from 'solid-js';

import MdiHomeVariantIcon from '@iconify/icons-mdi/home-variant';
import { useNavigate } from '@solidjs/router';

import type { IconButtonProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type HomeButtonProps = IconButtonProps;

const HomeButton: Component<HomeButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () => navigate('/');

  return (
    <CombinedTooltip
      as={IconButton}
      icon={MdiHomeVariantIcon}
      label={t('home.title')}
      onClick={handleClick}
      placement='right'
      variant='ghost'
      {...props}
    />
  );
};

export default HomeButton;
