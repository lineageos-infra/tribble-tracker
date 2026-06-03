<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import {
  banModel,
  banVersion,
  getInstallations,
  listBans,
  unbanModel,
  unbanVersion,
  type BannedItem,
  type InstallationFilters,
  type VersionRawTotalItem
} from '@/api/admin'
import Button from '@/components/ui/Button.vue'
import SnackBar from '@/components/ui/SnackBar.vue'
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
const modelInput = ref('')
const modelNote = ref('')
const modelBusy = ref(false)

async function submitBanModel() {
  const model = modelInput.value.trim()
  if (!model) return
  modelBusy.value = true
  try {
    await banModel(model, modelNote.value.trim() || undefined)
    notify(`Banned model "${model}"`)
    modelInput.value = ''
    modelNote.value = ''
    await loadBans()
  } catch (e) {
    notify((e as Error).message)
  } finally {
    modelBusy.value = false
  }
}

const versionInput = ref('')
const versionNote = ref('')
const versionBusy = ref(false)

async function submitBanVersion() {
  const version = versionInput.value.trim()
  if (!version) return
  versionBusy.value = true
  try {
    await banVersion(version, versionNote.value.trim() || undefined)
    notify(`Banned version "${version}"`)
    versionInput.value = ''
    versionNote.value = ''
    await loadBans()
  } catch (e) {
    notify((e as Error).message)
  } finally {
    versionBusy.value = false
  }
}

async function deleteBan(item: BannedItem) {
  try {
    if (item.model) {
      await unbanModel(item.model)
      notify(`Unbanned model "${item.model}"`)
    } else if (item.version) {
      await unbanVersion(item.version)
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
const installations = ref<VersionRawTotalItem[] | null>(null)
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
  <div class="mx-auto flex w-full container max-w-6xl flex-col gap-6">
    <header class="px-1">
      <h1 class="text-2xl font-bold text-on-surface sm:text-3xl">Admin</h1>
    </header>

    <div class="grid gap-6 lg:grid-cols-2">
      <section class="flex flex-col gap-4 rounded-3xl bg-surface-elevated p-5">
        <h2 class="text-lg font-medium text-on-surface">Ban model</h2>
        <TextField v-model="modelInput" label="Model" @submit="submitBanModel" />
        <TextField v-model="modelNote" label="Note (optional)" @submit="submitBanModel" />
        <Button
          class="self-start"
          :disabled="modelBusy || !modelInput.trim()"
          @click="submitBanModel"
        >
          <LoaderCircle v-if="modelBusy" class="size-4 animate-spin" />
          <Ban v-else class="size-4" />
          Ban model
        </Button>
      </section>

      <section class="flex flex-col gap-4 rounded-3xl bg-surface-elevated p-5">
        <h2 class="text-lg font-medium text-on-surface">Ban version</h2>
        <TextField v-model="versionInput" label="Version" @submit="submitBanVersion" />
        <TextField v-model="versionNote" label="Note (optional)" @submit="submitBanVersion" />
        <Button
          class="self-start"
          :disabled="versionBusy || !versionInput.trim()"
          @click="submitBanVersion"
        >
          <LoaderCircle v-if="versionBusy" class="size-4 animate-spin" />
          <Ban v-else class="size-4" />
          Ban version
        </Button>
      </section>
    </div>

    <section class="flex flex-col gap-4 rounded-3xl bg-surface-elevated p-5">
      <header class="flex items-baseline justify-between gap-2">
        <h2 class="text-lg font-medium text-on-surface">Banned items</h2>
        <button
          type="button"
          class="inline-flex items-center gap-1.5 rounded-full border border-outline-variant px-3 py-1.5 text-xs text-on-surface transition hover:border-brand-primary disabled:opacity-50"
          :disabled="bansLoading"
          @click="loadBans"
        >
          <RefreshCw class="size-3.5" :class="bansLoading && 'animate-spin'" />
          Refresh
        </button>
      </header>

      <p v-if="bansError" class="text-sm text-red-400">{{ bansError }}</p>
      <p v-else-if="bansLoading && !bans.length" class="text-sm text-on-surface-muted">Loading…</p>
      <p v-else-if="!bans.length" class="text-sm text-on-surface-muted">No banned items.</p>
      <div v-else class="overflow-x-auto">
        <table class="w-full text-left text-sm">
          <thead class="text-xs text-on-surface-muted">
            <tr>
              <th class="px-2 py-1 font-medium">Model</th>
              <th class="px-2 py-1 font-medium">Version</th>
              <th class="px-2 py-1 font-medium">Note</th>
              <th class="px-2 py-1 font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, i) in bans" :key="i" class="border-t border-outline-variant">
              <td class="px-2 py-1.5 text-on-surface">{{ item.model ?? '—' }}</td>
              <td class="px-2 py-1.5 text-on-surface">{{ item.version ?? '—' }}</td>
              <td class="px-2 py-1.5 text-on-surface-muted">{{ item.note ?? '—' }}</td>
              <td class="px-2 py-1.5 text-on-surface">
                <button
                  type="button"
                  class="inline-flex items-center gap-1.5 rounded-full border border-outline-variant px-3 py-1.5 text-xs text-on-surface transition hover:border-brand-primary disabled:opacity-50"
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
      </div>
    </section>

    <!-- Installations -->
    <section class="flex flex-col gap-4 rounded-3xl bg-surface-elevated p-5">
      <h2 class="text-lg font-medium text-on-surface">Installations</h2>
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <TextField
          v-for="key in FILTER_KEYS"
          :key="key"
          v-model="filters[key]"
          :label="key[0].toUpperCase() + key.slice(1)"
          @submit="queryInstallations"
        />
      </div>
      <Button class="self-start" :disabled="installationsBusy" @click="queryInstallations">
        <LoaderCircle v-if="installationsBusy" class="size-4 animate-spin" />
        <Search v-else class="size-4" />
        Query
      </Button>

      <p v-if="installationsError" class="text-sm text-red-400">{{ installationsError }}</p>
      <template v-else-if="installations">
        <p class="text-xs text-on-surface-muted">{{ sortedInstallations.length }} rows</p>
        <p v-if="!sortedInstallations.length" class="text-sm text-on-surface-muted">No results.</p>
        <div v-else class="-mx-1 overflow-y-auto pr-1" style="max-height: 480px">
          <ol class="space-y-0.5">
            <li
              v-for="row in sortedInstallations"
              :key="row.version_raw"
              class="flex items-baseline justify-between gap-4 px-2 py-1 font-mono text-sm"
            >
              <span class="text-on-surface">{{ row.version_raw }}</span>
              <span class="tabular-nums text-on-surface-muted">{{
                formatNumber(row.installations)
              }}</span>
            </li>
          </ol>
        </div>
      </template>
    </section>

    <SnackBar :message="snackbar" />
  </div>
</template>
