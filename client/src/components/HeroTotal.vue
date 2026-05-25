<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { formatNumber } from '@/utils/format'
import { TransitionPresets, useTransition } from '@vueuse/core'
import { toRef } from 'vue'

const props = defineProps<{
  value: number
}>()

const tweened = useTransition(toRef(props, 'value'), {
  duration: 900,
  transition: TransitionPresets.easeOutCubic
})
</script>

<template>
  <section
    class="flex flex-col items-center gap-2 rounded-3xl bg-brand-primary px-6 py-10 text-center sm:py-14"
  >
    <p class="text-5xl font-medium tabular-nums sm:text-6xl md:text-7xl">
      {{ formatNumber(Math.round(tweened)) }}
    </p>
    <p class="text-xs font-medium tracking-wider uppercase">Total active installs</p>
  </section>
</template>
