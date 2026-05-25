// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

import { getFilteredStats } from '@/api/client'
import type { StatsResponse } from '@/api/types'
import type { ActiveFilter } from '@/utils/filters'
import { useAsyncState } from '@vueuse/core'
import { watch, type Ref } from 'vue'

export function useStats(filter?: () => ActiveFilter[] | null) {
  const { state, isLoading, error, execute } = useAsyncState<
    StatsResponse | null,
    [ActiveFilter[] | null]
  >((f) => getFilteredStats(f ?? []), null, { immediate: false })

  watch(
    () => filter?.() ?? null,
    (f) => {
      void execute(0, f)
    },
    { immediate: true }
  )

  return { data: state, error: error as Ref<Error | null>, loading: isLoading }
}
