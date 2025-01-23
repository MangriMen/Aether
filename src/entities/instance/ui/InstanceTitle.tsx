import { Component, ComponentProps, splitProps } from 'solid-js';

export type InstanceTitleProps = ComponentProps<'div'> & {
  name?: string;
  loader?: string;
  gameVersion?: string;
};

export const InstanceTitle: Component<InstanceTitleProps> = (props) => {
  const [local, others] = splitProps(props, ['name', 'loader', 'gameVersion']);

  return (
    <div {...others}>
      <div>{local.name}</div>
      <div class='text-sm capitalize text-muted-foreground'>
        <span>{local.loader}</span>
        &nbsp;
        <span>{local.gameVersion}</span>
      </div>
    </div>
  );
};
