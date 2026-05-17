import { splitProps, type Component, type ComponentProps } from 'solid-js';

import type { Java } from '@/entities/java';

import { useJavaVersionActions } from '../../lib';
import { JavaVersionEntry } from './JavaVersionEntry';

export type JavaVersionEntryControlledProps = ComponentProps<'div'> & {
  version: string;
  java: Java | undefined;
};

export const JavaVersionEntryControlled: Component<
  JavaVersionEntryControlledProps
> = (props) => {
  const [local, others] = splitProps(props, ['version', 'java']);

  const actions = useJavaVersionActions();

  return (
    <JavaVersionEntry
      version={local.version}
      path={local.java?.path ?? ''}
      onInstallRecommended={() => actions.installRecommended?.(local.version)}
      // onTest={() => actions.test?.(local.version, local.java?.path ?? '')}
      isInstalling={actions.isInstalling()}
      // isTesting={actions.isTesting()}
      // isTestingFailed={actions.isTestingFailed()}
      {...others}
    />
  );
};
