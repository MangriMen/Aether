import type { Component, ComponentProps } from 'solid-js';

import { useNavigate } from '@solidjs/router';
import IconMdiPackage from '~icons/mdi/package-variant';

import { CONTENT_ROUTES } from '@/features/content';
import { useTranslation } from '@/shared/i18n';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';

export type ContentButtonProps = ComponentProps<'div'>;

export const ContentButton: Component<ContentButtonProps> = (props) => {
  const navigate = useNavigate();
  const [{ t }] = useTranslation();

  const handleClick = () => navigate(CONTENT_ROUTES.CONTENT);

  return (
    <CombinedTooltip
      label={t('content.title')}
      placement='right'
      as={IconButton}
      variant='ghost'
      icon={IconMdiPackage}
      onClick={handleClick}
      {...props}
    />
  );
};
