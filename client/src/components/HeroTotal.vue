<script setup lang="ts">
import { formatNumber } from '@/utils/format'
import { TransitionPresets, useTransition } from '@vueuse/core'
import { computed, toRef } from 'vue'

const props = defineProps<{
  label: string
  value: number
  sublabel?: string
}>()

const tweened = useTransition(toRef(props, 'value'), {
  duration: 900,
  transition: TransitionPresets.easeOutCubic
})

const display = computed(() => formatNumber(Math.round(tweened.value)))
</script>

<template>
  <section
    class="flex flex-col items-center gap-2 rounded-3xl border border-outline-variant bg-surface-elevated px-6 py-10 text-center sm:py-14"
  >
    <p class="text-xs font-medium tracking-wider text-on-surface-muted uppercase">
      {{ label }}
    </p>
    <p class="text-5xl font-medium tabular-nums text-on-surface sm:text-6xl md:text-7xl">
      {{ display }}
    </p>
  </section>
</template>
