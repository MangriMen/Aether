import { type Component, type ComponentProps, splitProps } from 'solid-js';

import { cn, useDeveloperModeCounter } from '@/shared/lib';

import { AppVersion } from './AppVersion';
import { OsVersion } from './OsVersion';

export type VersionInfoProps = ComponentProps<'div'>;

export const VersionInfo: Component<VersionInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [counter, handleClickLogo] = useDeveloperModeCounter();

  return (
    <div
      class={cn(
        'inline-flex flex-col text-muted-foreground rounded-md select-none',
        local.class,
      )}
      {...others}
    >
      <span>
        <AppVersion class='w-max cursor-pointer' onClick={handleClickLogo} />
        &nbsp;
        {Array(counter() + 1).join('.')}
      </span>
      <OsVersion />
    </div>
  );
};
