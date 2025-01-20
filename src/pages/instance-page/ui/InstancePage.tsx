import MdiClockIcon from '@iconify/icons-mdi/clock';
import { Icon } from '@iconify-icon/solid';
// eslint-disable-next-line import/named
import { RouteSectionProps } from '@solidjs/router';
import {
  Component,
  ComponentProps,
  createMemo,
  Show,
  useContext,
} from 'solid-js';

import { Separator } from '@/shared/ui';

import {
  useMappedInstances,
  InstanceActionButton,
  InstanceImage,
  useInstanceActions,
  RunningInstancesContext,
  formatTimePlayedHumanized,
  formatTimePlayed,
} from '@/entities/instance';

export type InstancePageProps = ComponentProps<'div'> & RouteSectionProps;

export const InstancePage: Component<InstancePageProps> = (props) => {
  const id = createMemo(() => props.params.id);

  const mappedInstances = useMappedInstances();

  const instance = createMemo(() => mappedInstances()?.[id()]);

  const [context, { get }] = useContext(RunningInstancesContext);
  const runningInstance = createMemo(() => get(context, id()));

  const { handleInstanceLaunch, handleInstanceStop } = useInstanceActions();

  const lastPlayedDate = createMemo(() => {
    const lastPlayed = instance()?.lastPlayed;
    return lastPlayed ? new Date(Date.parse(lastPlayed)) : undefined;
  });

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...props}>
      <Show when={instance()} fallback={<span>Instance not found</span>}>
        {(instance) => (
          <>
            <div class='flex gap-4'>
              <InstanceImage src={instance().iconPath} />
              <div class='flex flex-col gap-2'>
                <span class='text-2xl font-bold'>{instance().name}</span>
                <span class='capitalize text-muted-foreground'>
                  {instance().loader} {instance().loaderVersion}
                </span>
                <span class='inline-flex gap-2'>
                  <span
                    class='mt-auto inline-flex items-center gap-1 text-muted-foreground'
                    title={formatTimePlayed(instance().timePlayed)}
                  >
                    <Icon icon={MdiClockIcon} />
                    {formatTimePlayedHumanized(instance().timePlayed)}
                  </span>
                  <Separator orientation='vertical' />
                  <span class='mt-auto inline-flex items-center gap-1 text-muted-foreground'>
                    Last played:&nbsp;
                    {lastPlayedDate()?.toLocaleString() ?? 'Never'}
                  </span>
                </span>
              </div>
              <div class='ml-auto'>
                <InstanceActionButton
                  class='aspect-square p-2'
                  installStage={instance().installStage}
                  isRunning={runningInstance()?.isRunning}
                  isLoading={runningInstance()?.isLoading}
                  onLaunchClick={() => handleInstanceLaunch(instance())}
                  onStopClick={() => handleInstanceStop(instance())}
                />
              </div>
            </div>
            <Separator />
          </>
        )}
      </Show>
    </div>
  );
};
