import MdiArrowLeftIcon from '@iconify/icons-mdi/arrow-left';
import { useNavigate } from '@solidjs/router';
import { type Component, type ComponentProps, splitProps } from 'solid-js';

import type { Instance } from '@/entities/instances';

import { InstanceGameVersion } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { Button, Image } from '@/shared/ui';

export type InstanceInfoProps = {
  instance: Instance;
} & ComponentProps<'div'>;

export const InstanceInfo: Component<InstanceInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const navigate = useNavigate();

  const [{ t }] = useTranslation();

  const handleBackToInstance = () => {
    navigate(`/instances/${encodeURIComponent(local.instance.id)}`);
  };

  return (
    <div class={cn('flex gap-2 items-center', local.class)} {...others}>
      <Image class='h-14 w-max' />
      <div class='flex flex-col justify-between text-muted-foreground'>
        <span class='text-lg font-bold text-foreground'>
          {local.instance.name}
        </span>
        <InstanceGameVersion
          class='font-medium'
          gameVersion={local.instance.gameVersion}
          loader={local.instance.loader}
        />
      </div>
      <Button
        class='ml-auto'
        leadingIcon={MdiArrowLeftIcon}
        onClick={handleBackToInstance}
        variant='secondary'
      >
        {t('content.backToInstance')}
      </Button>
    </div>
  );
};
