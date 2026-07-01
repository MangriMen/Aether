import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn, useDeveloperModeCounter } from '@/shared/lib';

import { AppVersion } from './AppVersion';
import { OsVersion } from './OsVersion';

export type VersionInfoProps = ComponentProps<'div'>;

export const VersionInfo: Component<VersionInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [, handleClickLogo] = useDeveloperModeCounter();

  return (
    <div
      class={cn(
        'inline-flex flex-col rounded-md text-muted-foreground select-none',
        local.class,
      )}
      {...others}
    >
      <span>
        <AppVersion class='w-max' onClick={handleClickLogo} />
      </span>
      <OsVersion />
    </div>
  );
};
