<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
defineProps<{
  type?: 'button' | 'submit'
  disabled?: boolean
}>()
</script>

<template>
  <button
    :type="type ?? 'button'"
    :disabled="disabled"
    class="group bg-brand-primary text-brand-light relative inline-flex items-center justify-center overflow-hidden rounded-[1.25rem] px-5 py-2.5 text-sm font-medium transition-all duration-200 ease-out hover:rounded-lg active:rounded-lg disabled:pointer-events-none disabled:opacity-50"
  >
    <span
      class="bg-brand-light absolute inset-0 opacity-0 transition-opacity group-hover:opacity-10 group-active:opacity-20"
      aria-hidden="true"
    />
    <span class="relative z-10 inline-flex items-center gap-2">
      <slot />
    </span>
  </button>
</template>
