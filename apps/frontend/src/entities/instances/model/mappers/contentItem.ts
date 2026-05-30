import type {
  ContentSearchParamsDto,
  ContentSearchResultDto,
  ContentVersionDto,
} from '../../api';
import type {
  ContentSearchParams,
  ContentSearchResponse,
  ContentVersion,
} from '../contentItem';

export const contentSearchResultDtoToContentSearchResult = (
  dto: ContentSearchResultDto,
): ContentSearchResponse => ({
  ...dto,
  page: Number.parseInt(dto.page),
  pageSize: Number.parseInt(dto.pageSize),
  pageCount: Number.parseInt(dto.pageCount),
});

export const contentSearchParamsToContentSearchParamsDto = (
  dto: ContentSearchParams,
): ContentSearchParamsDto => ({
  ...dto,
  page: dto.page.toString(),
  pageSize: dto.pageSize.toString(),
});

export const contentVersionDtoToContentVersion = (
  dto: ContentVersionDto,
): ContentVersion => ({
  ...dto,
  downloads: Number.parseInt(dto.downloads),
});
