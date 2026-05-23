<script setup lang="ts">
import AppFooter from './AppFooter.vue'
import TopAppBar from './TopAppBar.vue'
</script>

<template>
  <div class="flex min-h-dvh flex-col bg-surface text-on-surface">
    <TopAppBar />
    <main class="flex-1 px-4 py-6 sm:px-8 sm:py-10">
      <slot />
    </main>
    <AppFooter />
  </div>
</template>
