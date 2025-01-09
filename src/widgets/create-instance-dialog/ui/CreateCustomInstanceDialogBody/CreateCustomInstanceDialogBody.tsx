import { createAsync } from '@solidjs/router';
import {
  Component,
  createMemo,
  createSignal,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';
import { createStore } from 'solid-js/store';

import { cn } from '@/shared/lib';
import { Option } from '@/shared/model';
import {
  Button,
  Collapsible,
  CollapsibleContent,
  DialogFooter,
  Field,
  TextField,
  TextFieldInput,
  TextFieldLabel,
} from '@/shared/ui';

import { refetchInstances } from '@/entities/instance';
import {
  createMinecraftInstance,
  getLoaderVersionsManifest,
  getMinecraftVersionManifest,
  InstanceCreateDto,
  LoaderVersion,
  ModLoader,
  Version,
} from '@/entities/minecraft';

import { SelectGameVersion } from '@/features/select-game-version';
import { SelectLoaderChips } from '@/features/select-loader-chips';
import { SelectLoaderTypeChips } from '@/features/select-loader-version';
import { SelectSpecificLoaderVersion } from '@/features/select-specific-loader-version';

import {
  CreateCustomInstanceFormProps,
  CreateCustomInstanceDialogBodyProps,
} from '.';

const loaders: Option<ModLoader>[] = [
  { name: 'Vanilla', value: ModLoader.Vanilla },
  { name: 'Forge', value: ModLoader.Forge },
  { name: 'Fabric', value: ModLoader.Fabric },
  { name: 'Quilt', value: ModLoader.Quilt },
];

const loaderVersionTypes: Option[] = [
  {
    name: 'Stable',
    value: 'stable',
  },
  {
    name: 'Latest',
    value: 'latest',
  },
  {
    name: 'Other',
    value: 'other',
  },
];

export const CreateCustomInstanceDialogBody: Component<
  CreateCustomInstanceDialogBodyProps
> = (props) => {
  const version_manifest = createAsync(() => getMinecraftVersionManifest());

  const [isCreating, setIsCreating] = createSignal(false);

  const [isAdvanced, setIsAdvanced] = createSignal(false);

  const [local, others] = splitProps(props, ['class', 'onOpenChange']);

  const [fields, setFields] = createStore<CreateCustomInstanceFormProps>({
    name: undefined,
    gameVersion: undefined,
    loader: ModLoader.Vanilla,
    loaderType: 'stable',
    loaderVersion: undefined,
  });

  const fabricVersions = createAsync(() =>
    getLoaderVersionsManifest(ModLoader.Fabric),
  );
  const forgeVersions = createAsync(() =>
    getLoaderVersionsManifest(ModLoader.Forge),
  );
  const quiltVersions = createAsync(() =>
    getLoaderVersionsManifest(ModLoader.Quilt),
  );

  const game_versions = createMemo(() => {
    if (fields.loader === ModLoader.Vanilla) {
      return version_manifest()?.versions ?? [];
    }

    return (
      version_manifest()?.versions.filter((version) => {
        switch (fields.loader) {
          case ModLoader.Fabric:
            return fabricVersions()?.gameVersions.some(
              (x) => version.id === x.id,
            );
          case ModLoader.Forge:
            return forgeVersions()?.gameVersions.some(
              (x) => version.id === x.id,
            );
          case ModLoader.Quilt:
            return quiltVersions()?.gameVersions.some(
              (x) => version.id === x.id,
            );
        }
      }) ?? []
    );
  });

  const specificLoaderVersions = createMemo(() => {
    const gameVersion = fields.gameVersion;
    if (gameVersion === undefined) {
      return [];
    }

    switch (fields.loader) {
      case ModLoader.Fabric:
        return fabricVersions()?.gameVersions[0].loaders ?? [];
      case ModLoader.Forge:
        return (
          forgeVersions()?.gameVersions.find(
            (item) => item.id === gameVersion.id,
          )?.loaders ?? []
        );
      case ModLoader.Quilt:
        return quiltVersions()?.gameVersions[0].loaders ?? [];
      default:
        return [];
    }
  });

  const handleSubmit = async (e: SubmitEvent) => {
    e.preventDefault();

    const data = new FormData(e.target as HTMLFormElement);

    const dataObject = { ...fields, ...Object.fromEntries(data.entries()) };

    if (dataObject.gameVersion === undefined) {
      return;
    }

    if (dataObject.loader === undefined) {
      return;
    }

    if (dataObject.name === undefined) {
      return;
    }

    setIsCreating(true);

    const dto: InstanceCreateDto = {
      name: dataObject.name,
      gameVersion: dataObject.gameVersion.id,
      modLoader: dataObject.loader,
    };

    try {
      createMinecraftInstance(dto);
      props.onOpenChange?.(false);
      refetchInstances();
    } catch (e) {
      console.error(e);
    }

    setIsCreating(false);
  };

  return (
    <form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <TextField class='flex flex-col gap-2'>
        <TextFieldLabel for='name'>Name</TextFieldLabel>
        <TextFieldInput
          type='text'
          id='name'
          name='name'
          class='max-w-[36ch]'
          maxLength={32}
          required
          autocomplete='off'
        />
      </TextField>

      <Field label='Loader'>
        <SelectLoaderChips
          loaders={loaders}
          defaultValue={loaders[0].value}
          value={fields.loader}
          onChange={(value) => {
            if (
              value &&
              Object.values(ModLoader).includes(value as ModLoader)
            ) {
              setFields('loader', value as ModLoader);
              setFields('loaderVersion', undefined);
            }
          }}
        />
      </Field>

      <Field label='Game Version'>
        <SelectGameVersion
          class='max-w-[31.5ch]'
          placeholder='Select game version'
          advanced={isAdvanced()}
          value={fields.gameVersion}
          options={game_versions()}
          onChange={(value: Version | null) => {
            if (value) {
              setFields('gameVersion', value);
            }
          }}
        />
      </Field>

      <Collapsible open={isAdvanced() && fields.loader !== ModLoader.Vanilla}>
        <CollapsibleContent class='p-2'>
          <Field label='Loader Version'>
            <SelectLoaderTypeChips
              loaderTypes={loaderVersionTypes}
              value={fields.loaderType}
              onChange={(value) => {
                if (value) {
                  setFields('loaderType', value);
                }
              }}
            />
          </Field>

          <Show when={fields.loaderType === 'other'}>
            <Show
              when={fields.gameVersion !== undefined}
              fallback={
                <span class='italic'>
                  Select a game version before you select a loader version
                </span>
              }
            >
              <Field label='Select version'>
                <SelectSpecificLoaderVersion
                  value={fields.loaderVersion || null}
                  placeholder='Select loader version'
                  options={specificLoaderVersions()}
                  onChange={(value) => {
                    if (value) {
                      setFields('loaderVersion', value as LoaderVersion);
                    }
                  }}
                />
              </Field>
            </Show>
          </Show>
        </CollapsibleContent>
      </Collapsible>

      <DialogFooter>
        <Button
          class='mb-2 sm:mb-0 sm:mr-auto'
          size='sm'
          onClick={() => setIsAdvanced(!isAdvanced())}
        >
          <Switch>
            <Match when={isAdvanced()}>Hide advanced</Match>
            <Match when={!isAdvanced()}>Show advanced</Match>
          </Switch>
        </Button>

        <Button
          size='sm'
          variant='success'
          type='submit'
          disabled={isCreating()}
        >
          Create
        </Button>

        <Button size='sm' onClick={() => props.onOpenChange?.(false)}>
          Cancel
        </Button>
      </DialogFooter>
    </form>
  );
};
