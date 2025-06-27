import { splitProps, type Component, type ComponentProps } from 'solid-js';
import { AppVersion } from './AppVersion';
import { OsVersion } from './OsVersion';
import { cn, useDeveloperModeCounter } from '@/6_shared/lib';

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
