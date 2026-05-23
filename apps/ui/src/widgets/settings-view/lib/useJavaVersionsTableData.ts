import { createMemo, type Accessor } from 'solid-js';

import type { Java } from '@/entities/java';

import type { JavaVersion } from '../model';

const MAJOR_JAVA_VERSIONS_TO_DISPLAY = [25, 21, 17, 8];

export const useJavaVersionsTableData = (
  javaVersions: Accessor<Java[] | undefined>,
): Accessor<JavaVersion[]> => {
  const versionMap = createMemo(() => {
    const map = new Map<number, JavaVersion>();
    const list = javaVersions();

    if (list) {
      for (let i = 0; i < list.length; i++) {
        const item = list[i];
        if (item) {
          map.set(item.majorVersion, item);
        }
      }
    }
    return map;
  });

  const tableData = createMemo(() => {
    const currentMap = versionMap();

    return MAJOR_JAVA_VERSIONS_TO_DISPLAY.map<JavaVersion>((version) => {
      const installedJava = currentMap.get(version);

      return {
        majorVersion: version,
        version: installedJava?.version,
        path: installedJava?.path,
      };
    });
  });

  return tableData;
};
