import type { ComponentProps } from 'solid-js';

import { createMemo, splitProps, type Component } from 'solid-js';

import { useJavaList, type Java } from '@/entities/java';
import { cn } from '@/shared/lib';
import { DataTable } from '@/shared/ui';

import type { JavaVersion } from '../../model';

import { createJavaVersionsTable } from '../../lib';

const MAJOR_JAVA_VERSIONS_TO_DISPLAY = ['25', '21', '17', '8'];

export type JavaVersionsTableProps = ComponentProps<'div'>;

export const JavaVersionsTable: Component<JavaVersionsTableProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const javaVersions = useJavaList();

  const versionToJava = createMemo(() => {
    return javaVersions.data?.reduce(
      (acc, java) => {
        acc[java.majorVersion] = java;
        return acc;
      },
      {} as Record<string, Java | undefined>,
    );
  });

  const mappedJavaVersions = createMemo(() => {
    return MAJOR_JAVA_VERSIONS_TO_DISPLAY.map<JavaVersion>((version) => ({
      majorVersion: version,
      path: versionToJava()?.[version]?.path,
    }));
  });

  const table = createJavaVersionsTable({
    data: mappedJavaVersions,
  });

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <DataTable table={table} isLoading={javaVersions.isLoading} />
    </div>
  );
};
