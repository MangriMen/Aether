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
  CombinedTextField,
  DialogFooter,
  LabeledField,
} from '@/shared/ui';

import {
  IncludeSnapshotsCheckbox,
  refetchInstances,
  createMinecraftInstance,
  InstanceCreateDto,
} from '@/entities/instance';
import {
  getLoaderVersionsManifest,
  getMinecraftVersionManifest,
  loaderManifestToMapped,
  ModLoader,
} from '@/entities/minecraft';

import { SelectGameVersion } from '@/features/select-game-version';
import { LoaderChipsToggleGroup } from '@/features/select-loader-chips';
import { LoaderVersionTypeChipsToggleGroup } from '@/features/select-loader-version';
import { SelectSpecificLoaderVersion } from '@/features/select-specific-loader-version';

import {
  CreateCustomInstanceSchema,
  filterGameVersions,
  filterGameVersionsForLoader,
  getLoaderVersionsForGameVersion,
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
    validate: zodForm(CreateCustomInstanceSchema),
    initialValues: {
      loader: ModLoader.Vanilla,
      loaderVersionType: loaderVersionTypes[0].value,
    },
  });

  const [isCreating, setIsCreating] = createSignal(false);

  const [isAdvanced, setIsAdvanced] = createSignal(false);

  const [shouldIncludeSnapshots, setShouldIncludeSnapshots] =
    createSignal(false);

  const versionManifest = createAsync(() => getMinecraftVersionManifest());

  const forgeVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Forge)),
  );
  const fabricVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Fabric)),
  );
  const quiltVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Quilt)),
  );

  const loaderVersions = createMemo(() =>
    getLoaderVersionsForGameVersion(
      getValue(form, 'loader'),
      getValue(form, 'gameVersion'),
      {
        fabric: fabricVersions(),
        forge: forgeVersions(),
        quilt: quiltVersions(),
      },
    ),
  );

  const loaderGameVersions = createMemo(() =>
    filterGameVersionsForLoader(
      getValue(form, 'loader'),
      versionManifest()?.versions ?? [],
      {
        fabric: fabricVersions(),
        forge: forgeVersions(),
        quilt: quiltVersions(),
      },
    ),
  );

  const filteredGameVersions = createMemo(() =>
    filterGameVersions(loaderGameVersions(), shouldIncludeSnapshots()),
  );

  const handleSubmit: SubmitHandler<CreateCustomInstanceFormValues> = async (
    values,
  ) => {
    setIsCreating(true);

    const payload: InstanceCreateDto = {
      name: values.name,
      gameVersion: values.gameVersion,
      modLoader: values.loader,
      loaderVersion: values.loaderVersion,
    };

    try {
      props.onOpenChange?.(false);
      refetchInstances();
      await createMinecraftInstance(payload);
    } catch (e) {
      console.error(e);
    }

    setIsCreating(false);
  };

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      shouldActive
      {...others}
    >
      <Field name='name'>
        {(field, props) => (
          <CombinedTextField
            label='Name'
            value={field.value}
            errorMessage={field.error}
            inputProps={{
              class: 'max-w-[36ch]',
              maxLength: 32,
              autocomplete: 'off',
              type: 'text',
              ...props,
            }}
          />
        )}
      </Field>

      <LabeledField label='Loader'>
        <Field name='loader'>
          {(field) => (
            <LoaderChipsToggleGroup
              loaders={loaders}
              value={field.value}
              onChange={(value) => {
                setValue(form, 'loader', value as ModLoader);
                setValue(form, 'loaderVersion', undefined);
              }}
            />
          )}
        </Field>
      </LabeledField>

      <LabeledField label='Game Version'>
        <div class='flex items-start gap-2'>
          <Field name='gameVersion'>
            {(field, props) => (
              <SelectGameVersion
                class='max-w-[31.5ch]'
                placeholder='Select game version'
                options={filteredGameVersions()}
                errorMessage={field.error}
                {...props}
                onChange={(value) => {
                  if (value) {
                    setValue(form, 'gameVersion', value.id);
                  }
                }}
              />
            )}
          </Field>
          <IncludeSnapshotsCheckbox
            class='mt-2'
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
            <Field name='loaderVersionType'>
              {(field) => (
                <LoaderVersionTypeChipsToggleGroup
                  loaderTypes={loaderVersionTypes}
                  value={field.value}
                  onChange={(value) => {
                    if (value) {
                      setValue(form, 'loaderVersionType', value);
                    }
                  }}
                />
              )}
            </Field>
          </LabeledField>

          <Show when={getValue(form, 'loaderVersionType') === 'other'}>
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
                  {(field, props) => (
                    <SelectSpecificLoaderVersion
                      placeholder='Select loader version'
                      options={loaderVersions()}
                      errorMessage={field.error}
                      {...props}
                      onChange={(value) => {
                        if (value) {
                          setValue(form, 'loaderVersion', value.id);
                        }
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
          variant='secondary'
          onClick={() => setIsAdvanced(!isAdvanced())}
        >
          <Switch>
            <Match when={isAdvanced()}>Hide advanced</Match>
            <Match when={!isAdvanced()}>Show advanced</Match>
          </Switch>
        </Button>

        <Button variant='success' type='submit' disabled={isCreating()}>
          Create
        </Button>

        <Button variant='secondary' onClick={() => props.onOpenChange?.(false)}>
          Cancel
        </Button>
      </DialogFooter>
    </Form>
  );
};
