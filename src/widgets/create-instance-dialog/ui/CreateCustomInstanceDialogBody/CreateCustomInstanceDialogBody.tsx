import {
  createForm,
  getValue,
  setValue,
  SubmitHandler,
  zodForm,
} from '@modular-forms/solid';
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

import { cn } from '@/shared/lib';
import {
  Button,
  Collapsible,
  CollapsibleContent,
  DialogFooter,
  LabeledField,
  TextField,
  TextFieldErrorMessage,
  TextFieldInput,
  TextFieldLabel,
} from '@/shared/ui';

import {
  IncludeSnapshotsCheckbox,
  refetchInstances,
} from '@/entities/instance';
import {
  createMinecraftInstance,
  getLoaderVersionsManifest,
  getMinecraftVersionManifest,
  InstanceCreateDto,
  loaderManifestToMapped,
  ModLoader,
  Version,
  VersionType,
} from '@/entities/minecraft';

import { SelectGameVersion } from '@/features/select-game-version';
import { SelectLoaderChips } from '@/features/select-loader-chips';
import { SelectLoaderTypeChips } from '@/features/select-loader-version';
import { SelectSpecificLoaderVersion } from '@/features/select-specific-loader-version';

import {
  CreateCustomInstanceSchema,
  loaders,
  loaderVersionTypes,
} from '../../model';

import {
  CreateCustomInstanceFormValues,
  CreateCustomInstanceDialogBodyProps,
} from '.';

export const CreateCustomInstanceDialogBody: Component<
  CreateCustomInstanceDialogBodyProps
> = (props) => {
  const [local, others] = splitProps(props, ['class', 'onOpenChange']);

  const [form, { Form, Field }] = createForm<CreateCustomInstanceFormValues>({
    validate: zodForm(CreateCustomInstanceSchema._def.schema),
    initialValues: {
      loader: ModLoader.Vanilla,
      loaderType: 'stable',
    },
  });

  const versionManifest = createAsync(() => getMinecraftVersionManifest());

  const [isCreating, setIsCreating] = createSignal(false);

  const [isAdvanced, setIsAdvanced] = createSignal(false);

  const [shouldIncludeSnapshots, setShouldIncludeSnapshots] =
    createSignal(false);

  const fabricVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Fabric)),
  );
  const forgeVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Forge)),
  );
  const quiltVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Quilt)),
  );

  const loaderGameVersions = createMemo(() => {
    const loader = getValue(form, 'loader');
    const versions = versionManifest()?.versions ?? [];
    if (loader === ModLoader.Vanilla) {
      return versions;
    }

    return versions.filter((version) => {
      switch (loader) {
        case ModLoader.Fabric:
          return !!fabricVersions()?.gameVersions?.[version.id];
        case ModLoader.Forge:
          return !!forgeVersions()?.gameVersions?.[version.id];
        case ModLoader.Quilt:
          return !!quiltVersions()?.gameVersions?.[version.id];
      }
    });
  });

  const gameVersions = createMemo(() => {
    return loaderGameVersions().filter((version) => {
      const isRelease = version.type === VersionType.Release;
      const needIncludeSnapshots = shouldIncludeSnapshots();
      return isRelease || needIncludeSnapshots;
    });
  });

  const specificLoaderVersions = createMemo(() => {
    const gameVersion = getValue(form, 'gameVersion');
    if (gameVersion === undefined) {
      return [];
    }

    const loader = getValue(form, 'loader');
    const dummyReplaceString = '${modrinth.gameVersion}';
    switch (loader) {
      case ModLoader.Fabric:
        return fabricVersions()?.gameVersions[dummyReplaceString].loaders ?? [];
      case ModLoader.Forge:
        return forgeVersions()?.gameVersions[gameVersion]?.loaders ?? [];
      case ModLoader.Quilt:
        return quiltVersions()?.gameVersions[dummyReplaceString].loaders ?? [];
      default:
        return [];
    }
  });

  const handleSubmit: SubmitHandler<CreateCustomInstanceFormValues> = async (
    values,
  ) => {
    setIsCreating(true);

    try {
      // props.onOpenChange?.(false);
      refetchInstances();
      // await createMinecraftInstance(formValuesToDto(values));
    } catch (e) {
      console.error(e);
    }

    console.log(values);
    setIsCreating(false);
  };

  const formValuesToDto = (
    values: CreateCustomInstanceFormValues,
  ): InstanceCreateDto => ({
    name: values.name,
    gameVersion: values.gameVersion,
    modLoader: values.loader,
    loaderVersion: values.loaderVersion,
  });

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      shouldActive
      {...others}
    >
      <button onClick={() => setValue(form, 'loader', ModLoader.Quilt)}>
        Кнопка
      </button>
      <Field name='name'>
        {(field, props) => (
          <TextField
            validationState={field.error ? 'invalid' : 'valid'}
            class='flex flex-col gap-2'
          >
            <TextFieldLabel for='name'>Name</TextFieldLabel>
            <TextFieldInput
              id='name'
              class='max-w-[36ch]'
              required
              type='text'
              maxLength={32}
              autocomplete='off'
              value={field.value}
              {...props}
            />
            <TextFieldErrorMessage>{field.error}</TextFieldErrorMessage>
          </TextField>
        )}
      </Field>

      <LabeledField label='Loader'>
        <Field name='loader'>
          {(_field, props) => (
            <SelectLoaderChips
              loaders={loaders}
              defaultValue={loaders[0].value}
              {...props}
              onChange={(value) => {
                if (!value) {
                  return;
                }

                setValue(form, 'loader', value as ModLoader);
                setValue(form, 'loaderVersion', undefined);
              }}
            />
          )}
        </Field>
      </LabeledField>

      <LabeledField label='Game Version'>
        <div class='flex gap-2'>
          <Field name='gameVersion'>
            {(_field, props) => (
              <SelectGameVersion
                validationState={_field.error ? 'invalid' : 'valid'}
                class='max-w-[31.5ch]'
                placeholder='Select game version'
                options={gameVersions()}
                {...props}
                onChange={(value: Version | null) => {
                  if (!value) {
                    return;
                  }

                  setValue(form, 'gameVersion', value.id);
                }}
              />
            )}
          </Field>
          <IncludeSnapshotsCheckbox
            show={isAdvanced()}
            checked={shouldIncludeSnapshots()}
            onChange={setShouldIncludeSnapshots}
          />
        </div>
      </LabeledField>

      <Collapsible
        open={isAdvanced() && getValue(form, 'loader') !== ModLoader.Vanilla}
      >
        <CollapsibleContent class='p-2'>
          <LabeledField label='Loader Version'>
            <Field name='loaderType'>
              {(field, props) => (
                <SelectLoaderTypeChips
                  loaderTypes={loaderVersionTypes}
                  value={field.value}
                  {...props}
                  onChange={(value) => {
                    if (!value) {
                      return;
                    }

                    setValue(form, 'loaderType', value);
                  }}
                />
              )}
            </Field>
          </LabeledField>

          <Show when={getValue(form, 'loaderType') === 'other'}>
            <Show
              when={getValue(form, 'gameVersion') !== undefined}
              fallback={
                <span class='italic'>
                  Select a game version before you select a loader version
                </span>
              }
            >
              <LabeledField label='Select version'>
                <Field name='loaderVersion'>
                  {(_, props) => (
                    <SelectSpecificLoaderVersion
                      placeholder='Select loader version'
                      options={specificLoaderVersions()}
                      {...props}
                      onChange={(value) => {
                        if (!value) {
                          return;
                        }

                        setValue(form, 'loaderVersion', value.id);
                      }}
                    />
                  )}
                </Field>
              </LabeledField>
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
    </Form>
  );
};
