import type { ComponentProps } from 'solid-js';

import { createMemo, For, splitProps, type Component } from 'solid-js';

import { useJavaList, type Java } from '@/entities/java';
import { cn } from '@/shared/lib';
import { DelayedShow, SkeletonList } from '@/shared/ui';

import { JavaVersionEntryControlled } from './JavaVersionEntryControlled';

const MAJOR_JAVA_VERSIONS_TO_DISPLAY = ['25', '21', '17', '8'];

export type JavaVersionsListProps = ComponentProps<'div'>;

export const JavaVersionsList: Component<JavaVersionsListProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const javaVersions = useJavaList();

  const versionToJava = createMemo(() => {
    return javaVersions.data?.reduce(
      (acc, java) => {
        acc[java.major_version] = java;
        return acc;
      },
      {} as Record<string, Java | undefined>,
    );
  });

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <DelayedShow
        when={!javaVersions.isLoading && Boolean(javaVersions?.data?.length)}
        fallback={
          <SkeletonList class='w-full rounded-md' height={128} itemsCount={3} />
        }
      >
        <For each={MAJOR_JAVA_VERSIONS_TO_DISPLAY}>
          {(version) => (
            <JavaVersionEntryControlled
              version={version}
              java={versionToJava()?.[version]}
            />
          )}
        </For>
      </DelayedShow>
    </div>
  );
};
