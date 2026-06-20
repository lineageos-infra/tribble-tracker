<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import {
  banModels,
  banVersions,
  getInstallations,
  listBans,
  unbanModels,
  unbanVersions,
  type BannedItem,
  type InstallationFilters,
  type TotalInstallationsItem
} from '@/api/admin'
import SnackBar from '@/components/ui/SnackBar.vue'
import SubmitButton from '@/components/ui/SubmitButton.vue'
import TextArea from '@/components/ui/TextArea.vue'
import TextField from '@/components/ui/TextField.vue'
import { formatNumber } from '@/utils/format'
import { Ban, CircleX, LoaderCircle, RefreshCw, Search } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'

const snackbar = ref<string | null>(null)
let snackbarTimer: ReturnType<typeof setTimeout> | undefined

function notify(message: string) {
  snackbar.value = message
  clearTimeout(snackbarTimer)
  snackbarTimer = setTimeout(() => (snackbar.value = null), 4000)
}

const bans = ref<BannedItem[]>([])
const bansLoading = ref(false)
const bansError = ref<string | null>(null)
const totalAffectedInstallations = computed(() =>
  bans.value.reduce((sum, item) => sum + (item.affected_installations ?? 0), 0)
)

async function loadBans() {
  bansLoading.value = true
  bansError.value = null
  try {
    bans.value = await listBans()
  } catch (e) {
    bansError.value = (e as Error).message
  } finally {
    bansLoading.value = false
  }
}

// --- Ban model / version -----------------------------------------------------
const modelsInput = ref('')
const modelsNote = ref('')
const modelsBusy = ref(false)

async function submitBanModels() {
  const models = modelsInput.value.split('\n').filter(Boolean)
  if (!models) return
  modelsBusy.value = true
  try {
    await banModels(models, modelsNote.value.trim() || undefined)
    notify(`Banned models: ${models.join(', ')}`)
    modelsInput.value = ''
    modelsNote.value = ''
    await loadBans()
  } catch (e) {
    notify((e as Error).message)
  } finally {
    modelsBusy.value = false
  }
}

const versionsInput = ref('')
const versionsNote = ref('')
const versionsBusy = ref(false)

async function submitBanVersions() {
  const versions = versionsInput.value.split('\n').filter(Boolean)
  if (!versions) return
  versionsBusy.value = true
  try {
    await banVersions(versions, versionsNote.value.trim() || undefined)
    notify(`Banned versions: ${versions.join(', ')}`)
    versionsInput.value = ''
    versionsNote.value = ''
    await loadBans()
  } catch (e) {
    notify((e as Error).message)
  } finally {
    versionsBusy.value = false
  }
}

async function deleteBan(item: BannedItem) {
  try {
    if (item.model) {
      await unbanModels([item.model])
      notify(`Unbanned model "${item.model}"`)
    } else if (item.version) {
      await unbanVersions([item.version])
      notify(`Unbanned version "${item.version}"`)
    }
    await loadBans()
  } catch (e) {
    notify((e as Error).message)
  }
}

const filters = ref<Required<InstallationFilters>>({
  model: '',
  country: '',
  version: '',
  carrier: ''
})
const installations = ref<TotalInstallationsItem[] | null>(null)
const installationsBusy = ref(false)
const installationsError = ref<string | null>(null)

const FILTER_KEYS = ['model', 'country', 'version', 'carrier'] as const

const sortedInstallations = computed(() =>
  installations.value
    ? [...installations.value].sort((a, b) => b.installations - a.installations)
    : []
)

async function queryInstallations() {
  installationsBusy.value = true
  installationsError.value = null
  try {
    installations.value = await getInstallations(filters.value)
  } catch (e) {
    installationsError.value = (e as Error).message
    installations.value = null
  } finally {
    installationsBusy.value = false
  }
}

onMounted(loadBans)
</script>

<template>
  <div class="container mx-auto flex w-full max-w-6xl flex-col gap-6">
    <header class="px-1">
      <h1 class="text-on-surface text-2xl font-bold sm:text-3xl">Admin</h1>
    </header>

    <div class="grid gap-6 lg:grid-cols-2">
      <section class="bg-surface-elevated flex flex-col gap-4 rounded-3xl p-5">
        <h2 class="text-on-surface text-lg font-medium">Ban models</h2>
        <TextArea v-model="modelsInput" label="Models" />
        <TextField v-model="modelsNote" label="Note (optional)" @submit="submitBanModels" />
        <SubmitButton
          class="self-start"
          :disabled="modelsBusy || !modelsInput.trim()"
          @click="submitBanModels"
        >
          <LoaderCircle v-if="modelsBusy" class="size-4 animate-spin" />
          <Ban v-else class="size-4" />
          Ban models
        </SubmitButton>
      </section>

      <section class="bg-surface-elevated flex flex-col gap-4 rounded-3xl p-5">
        <h2 class="text-on-surface text-lg font-medium">Ban versions</h2>
        <TextArea v-model="versionsInput" label="Versions" />
        <TextField v-model="versionsNote" label="Note (optional)" @submit="submitBanVersions" />
        <SubmitButton
          class="self-start"
          :disabled="versionsBusy || !versionsInput.trim()"
          @click="submitBanVersions"
        >
          <LoaderCircle v-if="versionsBusy" class="size-4 animate-spin" />
          <Ban v-else class="size-4" />
          Ban versions
        </SubmitButton>
      </section>
    </div>

    <section class="bg-surface-elevated flex flex-col gap-4 rounded-3xl p-5">
      <header class="flex items-baseline justify-between gap-2">
        <h2 class="text-on-surface text-lg font-medium">Banned items</h2>
        <button
          type="button"
          class="border-outline-variant text-on-surface hover:border-brand-primary inline-flex items-center gap-1.5 rounded-full border px-3 py-1.5 text-xs transition disabled:opacity-50"
          :disabled="bansLoading"
          @click="loadBans"
        >
          <RefreshCw class="size-3.5" :class="bansLoading && 'animate-spin'" />
          Refresh
        </button>
      </header>

      <p v-if="bansError" class="text-sm text-red-400">{{ bansError }}</p>
      <p v-else-if="bansLoading && !bans.length" class="text-on-surface-muted text-sm">Loading…</p>
      <p v-else-if="!bans.length" class="text-on-surface-muted text-sm">No banned items.</p>
      <div v-else class="overflow-x-auto">
        <table class="w-full text-left text-sm">
          <thead class="text-on-surface-muted text-xs">
            <tr>
              <th class="px-2 py-1 font-medium">Model</th>
              <th class="px-2 py-1 font-medium">Version</th>
              <th class="px-2 py-1 font-medium">Note</th>
              <th class="px-2 py-1 font-medium">Affected Installations</th>
              <th class="px-2 py-1 font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, i) in bans" :key="i" class="border-outline-variant border-t">
              <td class="text-on-surface px-2 py-1.5">{{ item.model ?? '—' }}</td>
              <td class="text-on-surface px-2 py-1.5">{{ item.version ?? '—' }}</td>
              <td class="text-on-surface-muted px-2 py-1.5">{{ item.note ?? '—' }}</td>
              <td class="text-on-surface-muted px-2 py-1.5 tabular-nums">
                {{ formatNumber(item.affected_installations ?? 0) }}
              </td>
              <td class="text-on-surface px-2 py-1.5">
                <button
                  type="button"
                  class="border-outline-variant text-on-surface hover:border-brand-primary inline-flex items-center gap-1.5 rounded-full border px-3 py-1.5 text-xs transition disabled:opacity-50"
                  :disabled="bansLoading"
                  @click="deleteBan(item)"
                >
                  <CircleX class="size-3.5" />
                  Delete
                </button>
              </td>
            </tr>
          </tbody>
        </table>
        <p
          v-if="totalAffectedInstallations"
          class="text-on-surface-muted mt-2 text-center text-xs tabular-nums"
        >
          Total affected installations: {{ formatNumber(totalAffectedInstallations) }}
        </p>
      </div>
    </section>

    <!-- Installations -->
    <section class="bg-surface-elevated flex flex-col gap-4 rounded-3xl p-5">
      <h2 class="text-on-surface text-lg font-medium">Installations</h2>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <TextField
          v-for="key in FILTER_KEYS"
          :key="key"
          v-model="filters[key]"
          :label="key[0].toUpperCase() + key.slice(1)"
          @submit="queryInstallations"
        />
      </div>
      <SubmitButton class="self-start" :disabled="installationsBusy" @click="queryInstallations">
        <LoaderCircle v-if="installationsBusy" class="size-4 animate-spin" />
        <Search v-else class="size-4" />
        Query
      </SubmitButton>

      <p v-if="installationsError" class="text-sm text-red-400">{{ installationsError }}</p>
      <template v-else-if="installations">
        <p class="text-on-surface-muted text-xs">{{ sortedInstallations.length }} rows</p>
        <p v-if="!sortedInstallations.length" class="text-on-surface-muted text-sm">No results.</p>
        <div v-else class="-mx-1 overflow-y-auto pr-1" style="max-height: 480px">
          <table class="w-full text-left text-sm">
            <thead class="text-on-surface-muted text-xs">
              <tr>
                <th class="px-2 py-1 font-medium">Model</th>
                <th class="px-2 py-1 font-medium">Version</th>
                <th class="px-2 py-1 font-medium">Installations</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in sortedInstallations"
                :key="row.version_raw"
                class="border-outline-variant border-t"
              >
                <td class="text-on-surface px-2 py-1.5">{{ row.model }}</td>
                <td class="text-on-surface px-2 py-1.5">{{ row.version_raw }}</td>
                <td class="text-on-surface-muted px-2 py-1.5 tabular-nums">
                  {{ formatNumber(row.installations) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </section>

    <SnackBar :message="snackbar" />
  </div>
</template>
