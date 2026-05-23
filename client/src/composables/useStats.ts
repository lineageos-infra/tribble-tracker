// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

import { getFilteredStats, getStats } from '@/api/client'
import type { FilterColumn, StatsResponse } from '@/api/types'
import { useAsyncState } from '@vueuse/core'
import { computed, watch } from 'vue'

interface Filter {
  column: FilterColumn
  name: string
}

export function useStats(filter?: () => Filter | null) {
  const { state, isLoading, error, execute } = useAsyncState<StatsResponse | null, [Filter | null]>(
    (f) => (f ? getFilteredStats(f.column, f.name) : getStats()),
    null,
    { immediate: false }
  )

  watch(
    () => filter?.() ?? null,
    (f) => {
      void execute(0, f)
    },
    { immediate: true }
  )

  const normalizedError = computed(() => {
    const e = error.value
    if (e === null || e === undefined) return null
    if (e instanceof Error) return e
    return new Error(typeof e === 'string' ? e : JSON.stringify(e))
  })

  return { data: state, error: normalizedError, loading: isLoading }
}
