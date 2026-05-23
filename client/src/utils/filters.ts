// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

import type { FilterColumn } from '@/api/types'
import { FILTER_COLUMNS } from '@/api/types'
import type { LocationQueryRaw, RouteLocationNormalizedLoaded, RouteLocationRaw } from 'vue-router'

export interface ActiveFilter {
  column: FilterColumn
  name: string
}

export function activeFiltersFromRoute(route: RouteLocationNormalizedLoaded): ActiveFilter[] {
  return FILTER_COLUMNS
    .map((column) => ({ column, name: route.query[column] }))
    .filter((entry): entry is ActiveFilter => typeof entry.name === 'string' && entry.name.length > 0)
}

export function queryFromFilters(filters: ActiveFilter[]): LocationQueryRaw {
  const query: LocationQueryRaw = {}
  for (const filter of filters) {
    query[filter.column] = filter.name
  }
  return query
}

export function routeForFilterSelection(
  currentRoute: RouteLocationNormalizedLoaded,
  target: ActiveFilter
): RouteLocationRaw {
  const activeFilters = activeFiltersFromRoute(currentRoute)
  const nextFilters = activeFilters.filter((filter) => filter.column !== target.column)
  nextFilters.push(target)

  return {
    name: 'filter',
    query: queryFromFilters(nextFilters)
  }
}

export function routeForClearingFilter(
  currentRoute: RouteLocationNormalizedLoaded,
  target: ActiveFilter
): RouteLocationRaw {
  const activeFilters = activeFiltersFromRoute(currentRoute)
  const remaining = activeFilters.filter((filter) => filter.column !== target.column)
  return remaining.length
    ? { name: 'filter', query: queryFromFilters(remaining) }
    : { name: 'overview' }
}
