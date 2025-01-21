// eslint-disable-next-line import/named
import { RouteSectionProps, useLocation, useNavigate } from '@solidjs/router';
import {
  Component,
  ComponentProps,
  createMemo,
  lazy,
  Show,
  splitProps,
} from 'solid-js';

import { Separator } from '@/shared/ui';

import { useMappedInstances } from '@/entities/instance';

import { InstanceHeader } from './InstanceHeader';

const InstanceSettingsDialog = lazy(() =>
  import('@/widgets/instance-settings-dialog').then((m) => ({
    default: m.InstanceSettingsDialog,
  })),
);

export type InstancePageProps = ComponentProps<'div'> & RouteSectionProps;

export const InstancePage: Component<InstancePageProps> = (props) => {
  const [_local, others] = splitProps(props, ['params', 'location', 'data']);

  const navigate = useNavigate();
  const location = useLocation();

  const showSettings = createMemo(() => location.pathname.endsWith('settings'));

  const id = createMemo(() => props.params.id);

  const mappedInstances = useMappedInstances();

  const instance = createMemo(() => mappedInstances()?.[id()]);

  return (
    <>
      <div class='flex size-full flex-col gap-2 p-4' {...others}>
        <Show when={instance()} fallback={<span>Instance not found</span>}>
          {(instance) => (
            <>
              <InstanceHeader instance={instance()} />
              <Separator />
            </>
          )}
        </Show>
      </div>
      <Show when={instance()} fallback={<span>Instance not found</span>}>
        {(instance) => (
          <InstanceSettingsDialog
            open={showSettings()}
            onOpenChange={() => navigate(-1)}
            instance={instance()}
          />
        )}
      </Show>
    </>
  );
};
