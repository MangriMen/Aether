import type { Instance } from '@/entities/instances';
import { InstanceGameVersion } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { Button, Image } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import MdiArrowLeftIcon from '@iconify/icons-mdi/arrow-left';
import { useNavigate } from '@solidjs/router';
import { useTranslate } from '@/shared/model';

export type InstanceInfoProps = ComponentProps<'div'> & {
  instance: Instance;
};

export const InstanceInfo: Component<InstanceInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['instance', 'class']);

  const navigate = useNavigate();

  const [{ t }] = useTranslate();

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
          loader={local.instance.loader}
          gameVersion={local.instance.gameVersion}
        />
      </div>
      <Button
        class='ml-auto'
        leadingIcon={MdiArrowLeftIcon}
        variant='secondary'
        onClick={handleBackToInstance}
      >
        {t('content.backToInstance')}
      </Button>
    </div>
  );
};
