<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { formatNumber } from '@/utils/format'
import { TransitionPresets, useTransition } from '@vueuse/core'
import { computed, toRef } from 'vue'

const props = defineProps<{
  value: number
  official: number
}>()

const tweened = useTransition(toRef(props, 'value'), {
  duration: 900,
  transition: TransitionPresets.easeOutCubic
})

const unofficial = computed(() => Math.max(props.value - props.official, 0))
const officialPct = computed(() =>
  props.value > 0 ? Math.round((props.official / props.value) * 100) : 0
)
</script>

<template>
  <section
    class="bg-brand-primary flex flex-col items-center gap-2 rounded-3xl px-6 py-10 text-center text-white sm:py-14"
  >
    <p class="text-5xl font-medium tabular-nums sm:text-6xl md:text-7xl">
      {{ formatNumber(Math.round(tweened)) }}
    </p>
    <p class="text-xs font-medium tracking-wider uppercase">Total active installs</p>

    <div v-if="value > 0" class="mt-4 flex w-full max-w-md flex-col gap-2">
      <div class="flex h-2 overflow-hidden rounded-full bg-white/20">
        <div
          class="h-full bg-white transition-[width] duration-900 ease-out"
          :style="{ width: `${officialPct}%` }"
        />
      </div>
      <div class="flex justify-between text-xs font-medium tabular-nums">
        <span class="flex items-center gap-1.5">
          <span class="size-2 rounded-full bg-white" />
          Official {{ formatNumber(official) }} · {{ officialPct }}%
        </span>
        <span class="flex items-center gap-1.5 text-white/80">
          <span class="size-2 rounded-full bg-white/40" />
          Unofficial {{ formatNumber(unofficial) }} · {{ 100 - officialPct }}%
        </span>
      </div>
    </div>
  </section>
</template>
