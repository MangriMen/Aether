import type {
  EditInstanceDto,
  ImportInstanceDto,
  NewInstanceDto,
} from '../api';

export type NewInstance = NewInstanceDto;

export type EditInstance = EditInstanceDto;

export type EditInstanceSettings = Omit<EditInstance, 'name'>;

export type ImportInstance = ImportInstanceDto;

export const isEditInstanceSettingsEmpty = (dto: EditInstanceSettings) =>
  Object.values(dto).every((value) => value === undefined);

export const isEditInstanceEmpty = (dto: EditInstance) =>
  isEditInstanceSettingsEmpty(dto) && dto.name === undefined;
