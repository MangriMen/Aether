import MdiRightArrowIcon from '@iconify/icons-mdi/chevron-right';
import { Icon } from '@iconify-icon/solid';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { DialogHeader, DialogTitle, Image } from '@/shared/ui';

import type { Instance } from '@/entities/instances';

import { useTranslate } from '@/shared/model';

export type InstanceSettingsDialogHeaderProps = ComponentProps<'div'> & {
  instance: Instance;
};

const InstanceSettingsDialogHeader: Component<
  InstanceSettingsDialogHeaderProps
> = (props) => {
  const [local, others] = splitProps(props, ['instance']);

  const [{ t }] = useTranslate();

  return (
    <DialogHeader {...others}>
      <DialogTitle class='flex items-center gap-1 text-muted-foreground'>
        <div class='flex items-center gap-2'>
          <Image class='size-8' />
          <span>{local.instance.name}</span>
        </div>
        <Icon class='text-xl' icon={MdiRightArrowIcon} />
        <span class='text-foreground'>{t('instance.settings')}</span>
      </DialogTitle>
    </DialogHeader>
  );
};

export default InstanceSettingsDialogHeader;
