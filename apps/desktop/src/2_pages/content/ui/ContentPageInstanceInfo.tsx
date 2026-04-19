import { useNavigate } from '@solidjs/router';
import IconMdiArrowLeft from '~icons/mdi/arrow-left';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceHeaderInfo } from '@/pages/instance/ui/InstanceHeaderInfo';
import { ROUTES } from '@/shared/config';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Image } from '@/shared/ui';

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

    navigate(ROUTES.INSTANCE(local.instance.id));
  };

  return (
    <div class={cn('flex gap-3', local.class)} {...others}>
      <Image src={local.instance?.iconPath ?? undefined} />
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
