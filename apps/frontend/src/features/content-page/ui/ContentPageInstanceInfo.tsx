import { useNavigate } from '@solidjs/router';
import IconMdiArrowLeft from '~icons/mdi/arrow-left';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { Instance } from '@/features/instance';

import {
  InstanceHeaderInfo,
  InstanceIcon,
  INSTANCE_ROUTES,
} from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { Button } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

export type ContentPageInstanceInfoProps = ComponentProps<'div'> & {
  instance?: Instance;
};

export const ContentPageInstanceInfo: Component<
  ContentPageInstanceInfoProps
> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const navigate = useNavigate();

  const [{ t }] = useTranslation();

  const handleBackToInstance = () => {
    if (!local.instance?.id) {
      return;
    }

    navigate(INSTANCE_ROUTES.INSTANCE(local.instance.id));
  };

  return (
    <div class={cn('gap-3 flex', local.class)} {...others}>
      <InstanceIcon src={local.instance?.iconPath ?? undefined} />
      <InstanceHeaderInfo instance={local.instance} showTimePlayed={false} />
      <div class='ml-auto flex items-center'>
        <Button
          class='ml-auto min-w-max'
          leadingIcon={IconMdiArrowLeft}
          variant='secondary'
          onClick={handleBackToInstance}
          loading={!local.instance}
        >
          {t('content.backToInstance')}
        </Button>
      </div>
    </div>
  );
};
