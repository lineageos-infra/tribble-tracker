<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { useId } from 'vue'

const model = defineModel<string>({ required: true })

defineProps<{
  label: string
  type?: string
  supportingText?: string
}>()

const id = useId()
</script>

<template>
  <div>
    <div class="relative">
      <textarea
        :id="id"
        v-model="model"
        :type="type ?? 'text'"
        placeholder=" "
        class="peer border-outline bg-surface text-on-surface hover:bg-surface-hover focus:border-brand-primary field-sizing-content w-full rounded-t-lg border-b-2 px-3 pt-5 pb-1.5 text-sm transition-colors outline-none"
      />
      <label
        :for="id"
        class="text-on-surface-muted peer-focus:text-brand-primary pointer-events-none absolute top-1.5 left-3 origin-left text-xs transition-all duration-200 ease-out peer-placeholder-shown:top-1/2 peer-placeholder-shown:-translate-y-3.5 peer-placeholder-shown:text-sm peer-focus:top-1.5 peer-focus:translate-y-0 peer-focus:text-xs"
      >
        {{ label }}
      </label>
    </div>
    <p v-if="supportingText" class="text-on-surface-muted mt-1 px-4 text-xs">
      {{ supportingText }}
    </p>
  </div>
</template>
