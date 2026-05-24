<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { formatNumber } from '@/utils/format'
import { TransitionPresets, useTransition } from '@vueuse/core'
import { computed, toRef } from 'vue'

const props = defineProps<{
  label: string
  value: number
}>()

const tweened = useTransition(toRef(props, 'value'), {
  duration: 900,
  transition: TransitionPresets.easeOutCubic
})

const display = computed(() => formatNumber(Math.round(tweened.value)))
</script>

<template>
  <section
    class="flex flex-col items-center gap-2 rounded-3xl bg-brand-primary px-6 py-10 text-center sm:py-14"
  >
    <p class="text-5xl font-medium tabular-nums sm:text-6xl md:text-7xl">
      {{ display }}
    </p>
    <p class="text-xs font-medium tracking-wider uppercase">
      {{ label }}
    </p>
  </section>
</template>
