import type { JSX } from 'solid-js';

import { useQueries } from '@tanstack/solid-query';
import { createComputed, splitProps, type Component } from 'solid-js';
import { createStore } from 'solid-js/store';

import type {
  ContentFilters,
  ContentItem,
  InstallContentPayload,
  Instance,
} from '@/entities/instances';

import {
  instanceContentsQuery,
  useInstallContent,
  useInstances,
} from '@/entities/instances';

import type {
  ContentContextActions,
  ContentContextType,
  ContentContextValue,
} from '../model/contentContext';

import { ContentContext } from '../model/contentContext';
import { createInstalledContentIndexStore } from '../model/installedContentIndexStore';

export type ContentContextProviderProps = {
  children: JSX.Element;
  instanceId?: string;
  filters?: ContentFilters;
  providerId?: string;
  providerDataContentIdField?: string;
};

export const ContentContextProvider: Component<ContentContextProviderProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'instanceId',
    'filters',
    'providerId',
    'providerDataContentIdField',
  ]);

  const [installedContentIndex, setInstalledContentIndex] =
    createInstalledContentIndexStore();

  const [store, setStore] = createStore<ContentContextValue>({
    installedContentIndex: installedContentIndex,
    installingContentIds: {},
    get providerId() {
      return local.providerId;
    },
    get providerDataContentIdField() {
      return local.providerDataContentIdField;
    },
    get instanceId() {
      return local.instanceId;
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
    const field = store.providerDataContentIdField;

    if (!providerId || !field) {
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
        const contentId = content.update?.[providerId]?.[field];

        if (contentId === undefined || typeof contentId !== 'string') {
          continue;
        }

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

  const handleInstallContent = async (
    item: ContentItem,
    instanceId?: Instance['id'],
  ) => {
    const field = store.providerDataContentIdField;
    const providerData = item.providerData;

    if (!field || !providerData) {
      return;
    }

    const contentId = providerData[field];

    if (!contentId || typeof contentId !== 'string') {
      return;
    }

    const instance = instanceId
      ? instances.data?.find((instance) => instance.id === instanceId)
      : undefined;

    const providerId = store.providerId;
    const finalInstanceId = instance?.id ?? local.instanceId;
    const gameVersion =
      instance?.gameVersion ?? local.filters?.gameVersions?.[0];
    const loader = instance?.loader ?? local.filters?.loaders?.[0];

    if (!providerId || !finalInstanceId || !gameVersion) {
      return;
    }

    const payload: InstallContentPayload = {
      gameVersion: gameVersion,
      loader: loader,
      contentType: item.contentType,
      contentVersion: undefined,
      provider: providerId,
      providerData: item.providerData,
    };

    if (item.contentType !== 'mod') {
      payload.loader = undefined;
    }

    setStore('installingContentIds', contentId, true);

    try {
      await installContent({ id: finalInstanceId, payload });
    } catch {
      /* empty */
    } finally {
      setStore('installingContentIds', contentId, false);
    }
  };

  const actions: ContentContextActions = {
    installContent: handleInstallContent,
    createIsContentInstalling: (contentId) => {
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
    createIsContentInstalled: (contentId, instanceId) => {
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
  };

  const context: ContentContextType = [store, actions];

  return <ContentContext.Provider value={context} {...others} />;
};
