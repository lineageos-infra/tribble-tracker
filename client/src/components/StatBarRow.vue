<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import type { FilterColumn } from '@/api/types'
import { countryFlag, countryName, formatNumber } from '@/utils/format'
import { computed } from 'vue'

const props = defineProps<{
  rank: number
  column: FilterColumn
  name: string
  count: number
  max: number
}>()

const percent = computed(() => Math.max(0.5, (props.count / props.max) * 100))

const displayPrimary = computed(() => {
  if (props.column === 'country') {
    const full = countryName(props.name)
    return full ?? props.name
  }
  return props.name
})

const displaySecondary = computed(() => {
  if (props.column === 'country') return props.name.toUpperCase()
  return null
})

const flag = computed(() => (props.column === 'country' ? countryFlag(props.name) : null))
</script>

<template>
  <RouterLink
    :to="`/${column}/${encodeURIComponent(name)}`"
    class="group relative flex items-center gap-3 overflow-hidden rounded-xl px-3 py-2.5 transition-colors hover:bg-surface-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-primary"
  >
    <span
      class="absolute inset-y-1 left-1 rounded-lg bg-bar-track transition-[width] duration-500 ease-out group-hover:bg-bar-track/80"
      :style="{ width: `calc(${percent}% - 0.5rem)` }"
      aria-hidden="true"
    />
    <span
      class="relative w-6 shrink-0 text-right text-xs font-medium tabular-nums text-on-surface-muted"
    >
      {{ rank }}
    </span>
    <span v-if="flag" class="relative shrink-0 text-base leading-none" aria-hidden="true">
      {{ flag }}
    </span>
    <span class="relative flex min-w-0 flex-1 items-baseline gap-2">
      <span class="truncate text-sm font-medium text-on-surface">{{ displayPrimary }}</span>
      <span v-if="displaySecondary" class="shrink-0 text-xs text-on-surface-muted">
        {{ displaySecondary }}
      </span>
    </span>
    <span class="relative shrink-0 text-sm tabular-nums text-on-surface">
      {{ formatNumber(count) }}
    </span>
  </RouterLink>
</template>
