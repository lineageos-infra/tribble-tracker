<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import type { FilterColumn } from '@/api/types'
import { formatColumnLabel, formatFilterValue } from '@/utils/format'
import { X } from '@lucide/vue'
import type { RouteLocationRaw } from 'vue-router'

const props = defineProps<{
  column: FilterColumn
  name: string
  clearTo: RouteLocationRaw
}>()
</script>

<template>
  <div
    class="inline-flex items-center gap-2 rounded-full border border-brand-primary/30 bg-brand-primary/10 py-1.5 pr-1.5 pl-3 text-sm text-brand-primary"
  >
    <span class="font-medium">{{ formatFilterValue(props.column, props.name) }}</span>
    <span class="text-xs text-brand-primary/75">{{ formatColumnLabel(props.column) }}</span>
    <RouterLink
      :to="props.clearTo"
      class="grid h-6 w-6 place-items-center rounded-full text-brand-primary hover:bg-brand-primary/15 focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-primary"
      aria-label="Clear filter"
    >
      <X class="size-3.5" :stroke-width="2.5" />
    </RouterLink>
  </div>
</template>
