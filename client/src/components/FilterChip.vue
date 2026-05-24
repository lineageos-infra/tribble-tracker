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
  <RouterLink
    :to="props.clearTo"
    :aria-label="`Clear ${formatColumnLabel(props.column)} filter: ${formatFilterValue(props.column, props.name)}`"
    class="inline-flex h-8 items-center gap-2 rounded-lg bg-brand-light pr-2 pl-4 text-sm leading-5 font-medium text-brand-dark transition focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-primary hover:brightness-95 active:brightness-90 dark:bg-brand-dark dark:text-brand-light dark:hover:brightness-110 dark:active:brightness-125"
  >
    <span class="font-normal text-brand-dark/70 uppercase dark:text-brand-light/70">{{
      formatColumnLabel(props.column)
    }}</span>
    <span>{{ formatFilterValue(props.column, props.name) }}</span>
    <X class="size-4.5" :stroke-width="2" aria-hidden="true" />
  </RouterLink>
</template>
