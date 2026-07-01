import IconMdiChevronRight from '~icons/mdi/chevron-right';
// import IconMdiMenu from '~icons/mdi/menu';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { type Instance, InstanceIcon } from '@/entities/instances';
import { useTranslation } from '@/shared/model';
import { DialogHeader, DialogTitle } from '@/shared/ui';

export type InstanceSettingsDialogHeaderProps = ComponentProps<'div'> & {
  instance: Instance;
  isTabListOpen?: boolean;
  onTabListOpenChange?: (open: boolean) => void;
};

const InstanceSettingsDialogHeader: Component<
  InstanceSettingsDialogHeaderProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'instance',
    'isTabListOpen',
    'onTabListOpenChange',
  ]);

  const [{ t }] = useTranslation();

  return (
    <DialogHeader {...others}>
      <DialogTitle class='gap-2 pl-6 text-muted-foreground flex items-center'>
        {/* <IconButton
          class='z-50'
          variant='ghost'
          icon={IconMdiMenu}
          onClick={toggleTabList}
        /> */}
        <div class='gap-1 flex items-center'>
          <div class='gap-2 flex items-center'>
            <InstanceIcon
              class='size-8'
              src={local.instance.iconPath ?? undefined}
            />
            <span class='leading-8 line-clamp-1 [word-break:break-word]'>
              {local.instance.name}
            </span>
          </div>
          <IconMdiChevronRight />
          <span class='text-foreground'>{t('instance.settings')}</span>
        </div>
      </DialogTitle>
    </DialogHeader>
  );
};

export default InstanceSettingsDialogHeader;
