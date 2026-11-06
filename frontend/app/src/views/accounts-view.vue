<script setup lang="ts">
import { computed, ref } from 'vue'
import AccountCard from '../features/accounts/components/account-card.vue'
import AccountCreateForm from '../features/accounts/components/account-create-form.vue'
import AccountEditPanel from '../features/accounts/components/account-edit-panel.vue'
import PendingAccountCommands from '../features/accounts/components/pending-account-commands.vue'
import { useAccountListQuery } from '../features/accounts/accounts-feature'
const selectedId = ref<string | null>(null)
const createSession = ref(0)
const pendingRefresh = ref(0)
const readRefresh = ref(0)
const query = useAccountListQuery()
const accounts = computed(() => query.data.value ?? [])
const isLoading = query.isLoading
function startCreate() {
  selectedId.value = null
  createSession.value++
}
async function refreshAccounts() {
  await query.refetch()
  pendingRefresh.value++
  readRefresh.value++
}
</script>
<template>
<main class="page-shell accounts-page" aria-labelledby="accounts-title">
  <header class="page-header">
    <div>
      <p class="eyebrow">Finanzas personales y del hogar</p>
      <h1 id="accounts-title">Cuentas</h1>
      <p class="intro">Conserva cada saldo en su moneda y registra cuándo comenzó a formar parte de tus cuentas.</p>
    </div>
    <button type="button" class="primary-button" @click="startCreate">Nueva cuenta</button>
  </header>
  <p v-if="query.error.value" class="notice error-notice" role="alert">{{ query.error.value.message }}</p>
  <PendingAccountCommands :refresh-token="pendingRefresh" />
  <div class="accounts-layout">
    <section class="account-list" aria-labelledby="account-list-title">
      <div class="section-heading">
        <h2 id="account-list-title">Tus cuentas</h2>
        <div class="list-actions">
          <span v-if="!isLoading" class="count">{{ accounts.length }}</span>
          <button type="button" class="text-button" :disabled="isLoading" @click="refreshAccounts">Actualizar cuentas</button>
        </div>
      </div>
      <p v-if="isLoading" role="status" class="muted">Cargando cuentas…</p>
      <p v-else-if="!query.error.value && query.data.value && accounts.length === 0" class="empty-state">Todavía no tienes cuentas. Registra una para ver sus saldos aquí.</p>
      <ul v-else class="account-items">
        <li v-for="account in accounts" :key="account.id">
          <AccountCard :account="account" :selected="selectedId === account.id" @select="selectedId = account.id" />
        </li>
      </ul>
    </section>
    <AccountCreateForm v-if="!selectedId" :key="createSession" @settled="pendingRefresh++" />
    <AccountEditPanel v-else :key="selectedId" :account-id="selectedId" :refresh-token="readRefresh" @close="startCreate" @settled="pendingRefresh++" />
  </div>
</main>
</template>
<style scoped>
.accounts-page {
  padding-block: 8px 24px;
}

.page-header,
.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.page-header {
  align-items: end;
  margin-bottom: 32px;
}

h1,
h2,
p {
  margin-top: 0;
}

h1 {
  margin-bottom: 12px;
  font-size: clamp(36px, 5vw, 52px);
  letter-spacing: -0.055em;
}

h2 {
  margin-bottom: 0;
  font-size: 22px;
  letter-spacing: -0.035em;
}

.intro,
.muted,
.empty-state {
  color: var(--muted);
  line-height: 1.5;
}

.intro {
  max-width: 640px;
  margin-bottom: 0;
  font-size: 16px;
}

.eyebrow {
  margin-bottom: 8px;
}

.accounts-layout {
  display: grid;
  grid-template-columns: minmax(250px, 0.78fr) minmax(0, 1.5fr);
  align-items: start;
  gap: var(--island-gap);
}

.account-list {
  border: 1px solid var(--border);
  border-radius: var(--island-radius);
  background: var(--surface);
  box-shadow: var(--shadow-island);
}

.account-list {
  min-width: 0;
  padding: var(--island-padding);
}

.section-heading {
  margin-bottom: 18px;
}

.list-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.count {
  display: grid;
  min-width: 28px;
  height: 28px;
  place-items: center;
  border-radius: 999px;
  color: var(--muted);
  background: var(--accent-soft);
  font-size: 13px;
  font-weight: 700;
}

.account-items {
  display: grid;
  margin: 0;
  padding: 0;
  list-style: none;
}

.account-items {
  gap: 10px;
}

.primary-button {
  display: inline-flex;
  min-height: 44px;
  align-items: center;
  justify-content: center;
  padding: 10px 16px;
  border: 1px solid transparent;
  border-radius: var(--control-radius);
  font: inherit;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

.primary-button {
  color: var(--on-accent);
  background: var(--accent);
}

.primary-button:hover:not(:disabled) {
  background: var(--accent-hover);
}

button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.text-button {
  border: 0;
  padding: 7px 4px;
  color: var(--accent);
  background: transparent;
  font: inherit;
  font-size: 12px;
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 3px;
  cursor: pointer;
}

.notice {
  margin: 0 0 20px;
  padding: 12px 14px;
  border-radius: var(--control-radius);
  font-size: 14px;
}

.error-notice {
  color: var(--danger);
  background: var(--danger-bg);
}

.empty-state {
  margin-bottom: 0;
  font-size: 13px;
}

@media (max-width: 760px) {
.accounts-page {
    padding-top: 28px;
  }

.page-header {
    align-items: start;
    flex-direction: column;
  }

.accounts-layout {
    grid-template-columns: 1fr;
  }
}
</style>
