import { createStore } from 'solid-js/store';
import type { Instance, InstanceFile } from './instance';
import { getInstanceContents } from '../api';
import { createEffect, type Accessor } from 'solid-js';

export interface InstanceContentStoreType {
  map: Record<string, InstanceFile> | undefined;
  array: InstanceFile[] | undefined;
  loading: boolean;
  error: boolean;
}

export interface InstancesContentStoreType {
  content: Record<Instance['id'], InstanceContentStoreType>;

  initContent: (instanceId: Instance['id']) => void;
  updateContent: (
    instanceId: Instance['id'],
    content: Record<string, InstanceFile> | undefined,
    error?: boolean,
  ) => void;
  removeContent: (instanceId: Instance['id']) => void;
}

export const [instanceContentStore, setInstanceContentStore] =
  createStore<InstancesContentStoreType>({
    content: {},

    initContent: (instanceId: Instance['id']) => {
      setInstanceContentStore('content', instanceId, () => ({
        loading: false,
        error: false,
      }));
    },

    updateContent: (
      instanceId: Instance['id'],
      content: Record<string, InstanceFile> | undefined,
      error: boolean = false,
    ) => {
      setInstanceContentStore('content', instanceId, (prev) => ({
        ...prev,
        map: content,
        array: content ? Object.values(content) : undefined,
        loading: false,
        error,
      }));
    },

    removeContent: (instanceId: Instance['id']) => {
      setInstanceContentStore('content', (prev) => {
        const { [instanceId]: _, ...rest } = prev;
        return rest;
      });
    },
  });

const fetchContentForInstance = async (instanceId: Instance['id']) => {
  try {
    const contents = await getInstanceContents(instanceId);
    instanceContentStore.updateContent(instanceId, contents);
  } catch {
    instanceContentStore.updateContent(instanceId, undefined, true);
    console.error("Can't get instance contents");
  }
};

export const useInstanceContent = (instanceId: Accessor<Instance['id']>) => {
  if (!instanceContentStore.content[instanceId()]) {
    instanceContentStore.initContent(instanceId());
  }

  const [content, setContent] = createStore<InstanceContentStoreType>(
    instanceContentStore.content[instanceId()],
  );

  const updateContent = async (instanceId: Instance['id']) => {
    await fetchContentForInstance(instanceId);
    setContent(instanceContentStore.content[instanceId]);
  };

  createEffect(() => {
    if (content.map === undefined) {
      updateContent(instanceId());
    }
  });

  return content;
};

export const refetchInstanceContent = (instanceId: Instance['id']) => {
  fetchContentForInstance(instanceId);
};
