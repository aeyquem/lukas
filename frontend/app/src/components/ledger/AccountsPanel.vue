<script setup lang="ts">
import { computed, reactive, ref } from "vue";

import type {
  AccountDto,
  AccountType,
  CreateAccountRequest,
} from "@/api/ledger";
import { formatMoney } from "@/formatters";
import BaseAlert from "@/components/base/BaseAlert.vue";
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseButton from "@/components/base/BaseButton.vue";
import BaseCard from "@/components/base/BaseCard.vue";
import BaseField from "@/components/base/BaseField.vue";
import BaseSelect from "@/components/base/BaseSelect.vue";
import type { BaseSelectOption } from "@/components/base/BaseSelect.vue";
import BaseTextInput from "@/components/base/BaseTextInput.vue";
import { accountTypeLabel } from "@/ledger/labels";

type DrawerMode = "closed" | "add" | "edit";

const props = defineProps<{
  accounts: AccountDto[];
  hasLoadError: boolean;
  isCreating: boolean;
}>();

const emit = defineEmits<{
  "submit-account": [request: CreateAccountRequest];
}>();

const drawerMode = ref<DrawerMode>("closed");
const accountForm = reactive({
  displayName: "",
  type: "bank" as AccountType,
  currency: "CLP",
  openingBalance: "",
  currentBalance: "",
});
const accountTypeOptions: BaseSelectOption[] = [
  { label: "Banco", value: "bank" },
  { label: "Efectivo", value: "cash" },
  { label: "Tarjeta", value: "credit_card" },
];

const drawerTitle = computed(() =>
  drawerMode.value === "edit" ? "Editar cuenta" : "Agregar cuenta",
);

const isDrawerOpen = computed(() => drawerMode.value !== "closed");
const isEditMode = computed(() => drawerMode.value === "edit");

function openAddPanel() {
  resetForm();
  drawerMode.value = "add";
}

function openEditPanel(account: AccountDto) {
  accountForm.displayName = account.displayName;
  accountForm.type = account.type;
  accountForm.currency = account.currency;
  accountForm.openingBalance = "";
  accountForm.currentBalance = account.currentBalance;
  drawerMode.value = "edit";
}

function closePanel() {
  drawerMode.value = "closed";
}

function submitAccount() {
  if (drawerMode.value !== "add") {
    return;
  }

  emit("submit-account", {
    displayName: accountForm.displayName.trim(),
    type: accountForm.type,
    currency: accountForm.currency.trim().toUpperCase(),
    openingBalance: accountForm.openingBalance.trim() || null,
  });

  resetForm();
  drawerMode.value = "closed";
}

function resetForm() {
  accountForm.displayName = "";
  accountForm.type = "bank";
  accountForm.currency = "CLP";
  accountForm.openingBalance = "";
  accountForm.currentBalance = "";
}
</script>

<template>
  <BaseCard class="panel">
    <template #header>
      <div class="panel-heading">
        <div>
          <h2>Cuentas</h2>
          <span>Todas las cuentas registradas</span>
        </div>
        <div class="panel-actions">
          <BaseBadge>{{ props.accounts.length }}</BaseBadge>
          <BaseButton size="sm" type="button" @click="openAddPanel">
            Agregar cuenta
          </BaseButton>
        </div>
      </div>
    </template>

    <BaseAlert v-if="props.hasLoadError" class="form-alert" tone="danger">
      No se pudieron cargar las cuentas.
    </BaseAlert>

    <div class="accounts-table-wrap">
      <table class="accounts-table">
        <thead>
          <tr>
            <th scope="col">Nombre</th>
            <th scope="col">Tipo</th>
            <th scope="col">Moneda</th>
            <th scope="col">Saldo actual</th>
            <th scope="col">Acciones</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="account in props.accounts" :key="account.id">
            <td>
              <strong>{{ account.displayName }}</strong>
            </td>
            <td>{{ accountTypeLabel(account.type) }}</td>
            <td>{{ account.currency }}</td>
            <td class="numeric">
              {{ formatMoney(account.currentBalance, account.currency) }}
            </td>
            <td>
              <div class="row-actions">
                <BaseButton
                  size="sm"
                  type="button"
                  variant="secondary"
                  @click="openEditPanel(account)"
                >
                  Editar
                </BaseButton>
                <BaseButton
                  :disabled="true"
                  size="sm"
                  title="Cerrar cuenta requiere soporte de API"
                  type="button"
                  variant="secondary"
                >
                  Cerrar cuenta
                </BaseButton>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </BaseCard>

  <Teleport to="body">
    <div
      v-if="isDrawerOpen"
      class="account-drawer-backdrop"
      role="presentation"
      @click.self="closePanel"
    >
      <aside
        aria-labelledby="account-drawer-title"
        aria-modal="true"
        class="account-drawer"
        role="dialog"
      >
        <header class="account-drawer__header">
          <div>
            <p class="eyebrow">Cuenta</p>
            <h2 id="account-drawer-title">{{ drawerTitle }}</h2>
          </div>
          <BaseButton type="button" variant="secondary" @click="closePanel">
            Cerrar
          </BaseButton>
        </header>

        <form class="account-form" @submit.prevent="submitAccount">
          <BaseField input-id="account-name" label="Nombre">
            <BaseTextInput
              id="account-name"
              v-model="accountForm.displayName"
              required
              maxlength="120"
            />
          </BaseField>

          <BaseField input-id="account-type" label="Tipo">
            <BaseSelect
              id="account-type"
              v-model="accountForm.type"
              :options="accountTypeOptions"
            />
          </BaseField>

          <BaseField input-id="account-currency" label="Moneda">
            <BaseTextInput
              id="account-currency"
              v-model="accountForm.currency"
              required
              maxlength="3"
              pattern="[A-Z]{3}"
            />
          </BaseField>

          <BaseField
            v-if="isEditMode"
            input-id="current-balance"
            label="Saldo actual"
          >
            <BaseTextInput
              id="current-balance"
              :model-value="
                formatMoney(accountForm.currentBalance, accountForm.currency)
              "
              disabled
            />
          </BaseField>

          <BaseField
            v-else
            input-id="opening-balance"
            label="Saldo inicial"
          >
            <BaseTextInput
              id="opening-balance"
              v-model="accountForm.openingBalance"
              inputmode="decimal"
              placeholder="0"
            />
          </BaseField>

          <div class="account-form__actions">
            <BaseButton type="button" variant="secondary" @click="closePanel">
              Cancelar
            </BaseButton>
            <BaseButton
              :disabled="isEditMode"
              :loading="props.isCreating"
              type="submit"
            >
              {{ isEditMode ? "Guardar cambios" : "Crear cuenta" }}
            </BaseButton>
          </div>
        </form>
      </aside>
    </div>
  </Teleport>
</template>

<style scoped>
.panel-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-block-end: 0;
}

.panel-heading h2 {
  margin: 0;
  font-size: 1rem;
}

.panel-heading span {
  color: var(--p-text-muted-color);
  font-size: 0.875rem;
}

.panel-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.form-alert {
  margin-block-end: 1rem;
}

.accounts-table-wrap {
  overflow-x: auto;
}

.accounts-table {
  inline-size: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.accounts-table th,
.accounts-table td {
  border-block-end: 1px solid var(--p-content-border-color);
  padding: 0.75rem;
  text-align: start;
  vertical-align: middle;
}

.accounts-table th {
  color: var(--p-text-muted-color);
  font-size: 0.75rem;
  text-transform: uppercase;
}

.accounts-table strong {
  overflow-wrap: anywhere;
}

.numeric {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  white-space: nowrap;
}

.row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.account-form {
  display: grid;
  gap: 0.75rem;
}

.account-form__actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding-block-start: 0.5rem;
}

@media (max-width: 680px) {
  .panel-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .panel-actions {
    justify-content: space-between;
  }

}
</style>

<style>
.account-drawer-backdrop {
  position: fixed;
  z-index: 40;
  inset: 0;
  display: flex;
  justify-content: flex-end;
  background: rgb(0 0 0 / 0.32);
}

.account-drawer {
  display: grid;
  align-content: start;
  gap: 1.25rem;
  inline-size: min(28rem, 100%);
  block-size: 100dvh;
  overflow-y: auto;
  border-inline-start: 1px solid var(--p-content-border-color);
  background: var(--p-content-background);
  padding: 1.25rem;
  box-shadow: 0 24px 60px rgb(0 0 0 / 0.16);
}

.account-drawer__header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 1rem;
}

.account-drawer__header h2 {
  margin: 0;
  font-size: 1.2rem;
}

.account-drawer .eyebrow {
  margin: 0 0 0.25rem;
  color: var(--p-text-muted-color);
  font-size: 0.75rem;
  font-weight: 800;
  text-transform: uppercase;
}

@media (max-width: 680px) {
  .account-drawer {
    inline-size: 100%;
  }
}
</style>
