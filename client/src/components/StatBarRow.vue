<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import type { FilterColumn } from '@/api/types'
import { routeForFilterSelection } from '@/utils/filters'
import { formatFilterValue, formatNumber } from '@/utils/format'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const props = defineProps<{
  rank: number
  column: FilterColumn
  name: string
  count: number
  max: number
}>()

const percent = computed(() => Math.max(0.5, (props.count / props.max) * 100))

const route = useRoute()
const target = computed(() =>
  routeForFilterSelection(route, { column: props.column, name: props.name })
)
</script>

<template>
  <RouterLink
    :to="target"
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
    <span class="relative flex min-w-0 flex-1 items-baseline gap-2">
      <span class="truncate text-sm font-medium text-on-surface">
        {{ formatFilterValue(props.column, props.name) }}
      </span>
      <span v-if="props.column === 'country'" class="shrink-0 text-xs text-on-surface-muted">
        {{ props.name.toUpperCase() }}
      </span>
    </span>
    <span class="relative shrink-0 text-sm tabular-nums text-on-surface">
      {{ formatNumber(count) }}
    </span>
  </RouterLink>
</template>
