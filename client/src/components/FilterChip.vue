<script setup lang="ts">
import type { FilterColumn } from '@/api/types'
import { formatColumnLabel, formatFilterValue } from '@/utils/format'

const props = defineProps<{
  column: FilterColumn
  name: string
}>()
</script>

<template>
  <div
    class="inline-flex items-center gap-2 rounded-full border border-brand-primary/30 bg-brand-primary/10 py-1.5 pr-1.5 pl-3 text-sm text-brand-primary"
  >
    <span class="font-medium">{{ formatFilterValue(props.column, props.name) }}</span>
    <span class="text-xs text-brand-primary/75">{{ formatColumnLabel(props.column) }}</span>
    <RouterLink
      to="/"
      class="grid h-6 w-6 place-items-center rounded-full text-brand-primary hover:bg-brand-primary/15 focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-primary"
      aria-label="Clear filter"
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        class="h-3.5 w-3.5"
      >
        <path d="M6 6l12 12M18 6L6 18" stroke-linecap="round" />
      </svg>
    </RouterLink>
  </div>
</template>
