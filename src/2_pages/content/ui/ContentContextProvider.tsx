import type { JSX } from 'solid-js';

import { useQueries } from '@tanstack/solid-query';
import { createComputed, splitProps, type Component } from 'solid-js';
import { createStore } from 'solid-js/store';

import type {
  ContentFilters,
  ContentItem,
  InstallContentParams,
  Instance,
  ProviderId,
} from '@/entities/instances';

import {
  getContentIdFromUpdateInfo,
  instanceContentsQuery,
  isAtomicContentType,
  useInstallContent,
  useInstances,
} from '@/entities/instances';

import type {
  ContentContextActions,
  ContentContextType,
  ContentContextValue,
} from '../model/contentContext';
import type { ContentFiltersLock } from '../model/contentFiltersLock';

import { ContentContext } from '../model/contentContext';
import { createInstalledContentIndexStore } from '../model/installedContentIndexStore';

export type ContentContextProviderProps = {
  children: JSX.Element;
  instanceId: string | undefined;
  providerId: ProviderId | undefined;
  filters: ContentFilters | undefined;
  filtersLock: ContentFiltersLock | undefined;
  onFiltersChange?: (filters: ContentFilters) => void;
};

export const ContentContextProvider: Component<ContentContextProviderProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'providerId',
    'filters',
    'filtersLock',
    'onFiltersChange',
  ]);

  const [installedContentIndex, setInstalledContentIndex] =
    createInstalledContentIndexStore();

  const [store, setStore] = createStore<ContentContextValue>({
    installedContentIndex: installedContentIndex,
    installingContentIds: {},
    get instanceId() {
      return local.instanceId;
    },
    get providerId() {
      return local.providerId;
    },
    get filters() {
      return local.filters;
    },
    get filtersLock() {
      return local.filtersLock;
    },
  });

  const instances = useInstances();

  const instancesContents = useQueries(() => ({
    queries: (instances.data ?? []).map((instance) =>
      instanceContentsQuery(instance.id),
    ),
  }));

  createComputed(() => {
    const providerId = store.providerId;

    if (!providerId) {
      return;
    }

    if (!instances.data) {
      return;
    }

    const newContentMap: Record<string, string[]> = {};

    instancesContents.forEach((contents, index) => {
      if (!contents.data) {
        return;
      }

      const instance = instances.data[index];

      if (!instance) {
        return;
      }

      const instanceId = instance.id;

      for (const content of Object.values(contents.data)) {
        const contentId = getContentIdFromUpdateInfo(content, providerId);

        if (!newContentMap[contentId]) {
          newContentMap[contentId] = [];
        }

        if (!newContentMap[contentId].includes(instanceId)) {
          newContentMap[contentId].push(instanceId);
        }
      }
    });

    setInstalledContentIndex('content', newContentMap);
  });

  const { mutateAsync: installContent } = useInstallContent();

  const handleInstallAtomicContent = async (
    item: Pick<ContentItem, 'id' | 'contentType'>,
    instanceId?: Instance['id'],
    contentVersion?: string,
  ) => {
    const contentId = item.id;

    if (!contentId) {
      return;
    }

    const finalInstanceId = instanceId ?? local.instanceId;

    const instance = finalInstanceId
      ? instances.data?.find((instance) => instance.id === finalInstanceId)
      : undefined;

    const providerId = store.providerId;

    const gameVersion =
      instance?.gameVersion ?? local.filters?.gameVersions?.[0];
    const loader = instance?.loader ?? local.filters?.loaders?.[0];

    if (!providerId || !finalInstanceId || !gameVersion) {
      return;
    }

    if (!isAtomicContentType(item.contentType)) {
      return;
    }

    const payload: InstallContentParams = {
      type: 'atomic',
      data: {
        instanceId: finalInstanceId,
        gameVersion,
        loader,
        contentId: item.id,
        contentType: item.contentType,
        contentVersion,
        providerId,
      },
    };

    if (item.contentType !== 'mod') {
      payload.data.loader = undefined;
    }

    setStore('installingContentIds', contentId, true);

    try {
      await installContent(payload);
    } catch {
      /* empty */
    } finally {
      setStore('installingContentIds', contentId, false);
    }
  };

  const handleInstallModpack = async (
    item: Pick<ContentItem, 'id' | 'contentType'>,
  ) => {
    const contentId = item.id;

    if (!contentId) {
      return;
    }

    const providerId = store.providerId;

    if (!providerId) {
      return;
    }

    const payload: InstallContentParams = {
      type: 'modpack',
      data: {
        contentId,
        providerId,
      },
    };

    setStore('installingContentIds', contentId, true);

    try {
      await installContent(payload);
    } catch {
      // empty
    } finally {
      setStore('installingContentIds', contentId, false);
    }
  };

  const handleInstallContent = async (
    item: Pick<ContentItem, 'id' | 'contentType'>,
    instanceId?: Instance['id'],
    contentVersion?: string,
  ) => {
    const contentId = item.id;

    if (!contentId) {
      return;
    }

    if (item.contentType === 'modpack') {
      handleInstallModpack(item);
    } else {
      handleInstallAtomicContent(item, instanceId, contentVersion);
    }
  };

  const actions: ContentContextActions = {
    installContent: handleInstallContent,
    createIsInstalling: (contentId) => {
      // We return a standard accessor.
      // The linter complains because it's defined inside an object.
      // eslint-disable-next-line solid/reactivity
      return () => {
        const contentId_ = contentId();

        if (!contentId_) {
          return false;
        }

        // Accessing the store key here IS reactive when this returned function
        // is eventually called inside a component's JSX or createMemo.
        return store.installingContentIds[contentId_];
      };
    },
    createIsInstalled: (contentId, instanceId) => {
      return () => {
        const contentId_ = contentId();

        if (!contentId_) {
          return false;
        }

        const instanceId_ = instanceId();

        if (!instanceId_) {
          return false;
        }

        const instances = installedContentIndex.content[contentId_];

        if (!instances) {
          return false;
        }

        return instances.includes(instanceId_);
      };
    },
    getInstancesForContent: (contentId) =>
      installedContentIndex.content[contentId] ?? [],
    setFilters: (filters) => {
      local.onFiltersChange?.(filters);
    },
  };

  const context: ContentContextType = [store, actions];

  return <ContentContext.Provider value={context} {...others} />;
};
