export interface StatsResponse {
  model: Record<string, number>
  country: Record<string, number>
  version: Record<string, number>
  carrier: Record<string, number>
  total: number
}

export type FilterColumn = 'model' | 'country' | 'version' | 'carrier'

export const FILTER_COLUMNS: FilterColumn[] = ['model', 'country', 'version', 'carrier']

export function isFilterColumn(value: string): value is FilterColumn {
  return (FILTER_COLUMNS as string[]).includes(value)
}
