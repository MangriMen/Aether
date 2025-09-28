import type { DialogRootProps } from '@kobalte/core/dialog';
import type { FieldValues, SubmitHandler } from '@modular-forms/solid';
import type { Component } from 'solid-js';
import type { z } from 'zod';

import { createForm, getValue, setValue, zodForm } from '@modular-forms/solid';
import {
  createMemo,
  createSignal,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';

import type { NewInstance } from '@/entities/instances';
import type { LoaderVersion, Version } from '@/entities/minecraft';

import {
  IncludeSnapshotsCheckbox,
  useCreateInstance,
} from '@/entities/instances';
import {
  ModLoader,
  useLoaderVersionManifest,
  useMinecraftVersionManifest,
} from '@/entities/minecraft';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  Collapsible,
  CollapsibleContent,
  CombinedTextField,
  DialogFooter,
  LabeledField,
} from '@/shared/ui';

import { loaderManifestToMapped } from '../lib';
import {
  CreateCustomInstanceSchema,
  filterGameVersions,
  filterGameVersionsForLoader,
  getLoaderVersionsForGameVersion,
  LOADERS,
  LOADER_VERSION_TYPES,
} from '../model';
import { LoaderChipsToggleGroup } from './LoaderChipsToggleGroup';
import { LoaderVersionTypeChipsToggleGroup } from './LoaderVersionTypeChipsToggleGroup';
import { SelectGameVersion } from './SelectGameVersion';
import { SelectSpecificLoaderVersion } from './SelectSpecificLoaderVersion';

export type CreateCustomInstanceProps = { class?: string } & Pick<
  DialogRootProps,
  'onOpenChange'
>;

export type CreateCustomInstanceFormValues = FieldValues &
  z.infer<typeof CreateCustomInstanceSchema>;

export const CreateCustomInstance: Component<CreateCustomInstanceProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class', 'onOpenChange']);

  const [{ t }] = useTranslation();

  const [form, { Form, Field }] = createForm<CreateCustomInstanceFormValues>({
    validate: zodForm(CreateCustomInstanceSchema),
    initialValues: {
      loader: ModLoader.Vanilla,
      loaderVersionType: LOADER_VERSION_TYPES[0].value,
    },
  });

  const [isCreating, setIsCreating] = createSignal(false);

  const [isAdvanced, setIsAdvanced] = createSignal(false);

  const [shouldIncludeSnapshots, setShouldIncludeSnapshots] =
    createSignal(false);

  const versionManifest = useMinecraftVersionManifest();

  const forgeVersions = useLoaderVersionManifest(() => ModLoader.Forge);
  const fabricVersions = useLoaderVersionManifest(() => ModLoader.Fabric);
  const quiltVersions = useLoaderVersionManifest(() => ModLoader.Quilt);
  const neoforgeVersions = useLoaderVersionManifest(() => ModLoader.NeoForge);

  const forgeVersionsMapped = createMemo(() =>
    forgeVersions.data ? loaderManifestToMapped(forgeVersions.data) : undefined,
  );
  const fabricVersionsMapped = createMemo(() =>
    fabricVersions.data
      ? loaderManifestToMapped(fabricVersions.data)
      : undefined,
  );
  const quiltVersionsMapped = createMemo(() =>
    quiltVersions.data ? loaderManifestToMapped(quiltVersions.data) : undefined,
  );
  const neoforgeVersionsMapped = createMemo(() =>
    neoforgeVersions.data
      ? loaderManifestToMapped(neoforgeVersions.data)
      : undefined,
  );

  const loaderVersions = createMemo(() =>
    getLoaderVersionsForGameVersion(
      getValue(form, 'loader'),
      getValue(form, 'gameVersion'),
      {
        fabric: fabricVersionsMapped(),
        forge: forgeVersionsMapped(),
        quilt: quiltVersionsMapped(),
        neoforge: neoforgeVersionsMapped(),
      },
    ),
  );

  const loaderGameVersions = createMemo(() =>
    filterGameVersionsForLoader(
      getValue(form, 'loader'),
      versionManifest.data?.versions ?? [],
      {
        fabric: fabricVersionsMapped(),
        forge: forgeVersionsMapped(),
        quilt: quiltVersionsMapped(),
        neoforge: neoforgeVersionsMapped(),
      },
    ),
  );

  const filteredGameVersions = createMemo(() =>
    filterGameVersions(loaderGameVersions(), shouldIncludeSnapshots()),
  );

  const { mutateAsync: createInstance } = useCreateInstance();
  const handleSubmit: SubmitHandler<CreateCustomInstanceFormValues> = async (
    values,
  ) => {
    setIsCreating(true);

    const payload: NewInstance = {
      name: values.name,
      gameVersion: values.gameVersion,
      modLoader: values.loader,
      loaderVersion: values.loaderVersion,
    };

    props.onOpenChange?.(false);

    try {
      await createInstance(payload);
    } catch {
      /* empty */
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
            label={t('common.name')}
            value={field.value}
            errorMessage={field.error}
            inputProps={{
              class: 'max-w-[36ch]',
              maxLength: 32,
              type: 'text',
              ...props,
            }}
          />
        )}
      </Field>

      <LabeledField label={t('common.loader')}>
        <Field name='loader'>
          {(field) => (
            <LoaderChipsToggleGroup
              loaders={LOADERS}
              value={field.value}
              onChange={(value) => {
                setValue(form, 'loader', value as ModLoader);
                setValue(form, 'loaderVersion', undefined);
              }}
            />
          )}
        </Field>
      </LabeledField>

      <LabeledField label={t('common.gameVersion')}>
        <div class='flex items-start gap-2'>
          <Field name='gameVersion'>
            {(field, props) => (
              <SelectGameVersion
                class='max-w-[31.5ch]'
                placeholder={t('createInstance.gameVersionPlaceholder')}
                options={filteredGameVersions()}
                errorMessage={field.error}
                {...props}
                onChange={(value: Version | null) => {
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
          <LabeledField label={t('createInstance.loaderVersion')}>
            <Field name='loaderVersionType'>
              {(field) => (
                <LoaderVersionTypeChipsToggleGroup
                  loaderTypes={LOADER_VERSION_TYPES}
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
                  {t('createInstance.loaderVersionNoGameVersion')}
                </span>
              }
            >
              <LabeledField label={t('createInstance.loaderVersion')}>
                <Field name='loaderVersion'>
                  {(field, props) => (
                    <SelectSpecificLoaderVersion
                      placeholder={t('createInstance.loaderVersionPlaceholder')}
                      options={loaderVersions()}
                      errorMessage={field.error}
                      {...props}
                      onChange={(value: LoaderVersion | null) => {
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
            <Match when={isAdvanced()}>
              {t('createInstance.hideAdvanced')}
            </Match>
            <Match when={!isAdvanced()}>
              {t('createInstance.showAdvanced')}
            </Match>
          </Switch>
        </Button>

        <Button type='submit' disabled={isCreating()}>
          {t('common.create')}
        </Button>

        <Button variant='secondary' onClick={() => props.onOpenChange?.(false)}>
          {t('common.cancel')}
        </Button>
      </DialogFooter>
    </Form>
  );
};
