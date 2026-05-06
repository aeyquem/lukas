<script setup lang="ts">
import { reactive } from "vue";

import type {
  AccountDto,
  AccountType,
  CreateAccountRequest,
} from "../../api/ledger";
import { formatMoney } from "../../formatters";
import { accountTypeLabel } from "../../ledger/labels";
import BaseAlert from "../base/BaseAlert.vue";
import BaseBadge from "../base/BaseBadge.vue";
import BaseButton from "../base/BaseButton.vue";
import BaseCard from "../base/BaseCard.vue";
import BaseField from "../base/BaseField.vue";
import BaseSelect from "../base/BaseSelect.vue";
import BaseTextInput from "../base/BaseTextInput.vue";

const props = defineProps<{
  accounts: AccountDto[];
  hasLoadError: boolean;
  isCreating: boolean;
}>();

const emit = defineEmits<{
  "submit-account": [request: CreateAccountRequest];
}>();

const accountForm = reactive({
  displayName: "",
  type: "bank" as AccountType,
  currency: "CLP",
  openingBalance: "",
});

function submitAccount() {
  emit("submit-account", {
    displayName: accountForm.displayName.trim(),
    type: accountForm.type,
    currency: accountForm.currency.trim().toUpperCase(),
    openingBalance: accountForm.openingBalance.trim() || null,
  });

  accountForm.displayName = "";
  accountForm.openingBalance = "";
}
</script>

<template>
  <BaseCard class="panel">
    <template #header>
      <div class="panel-heading">
        <h2>Cuentas</h2>
        <BaseBadge>{{ props.accounts.length }}</BaseBadge>
      </div>
    </template>

    <form class="stacked-form" @submit.prevent="submitAccount">
      <BaseField input-id="account-name" label="Nombre">
        <BaseTextInput
          id="account-name"
          v-model="accountForm.displayName"
          required
          maxlength="120"
        />
      </BaseField>
      <BaseField input-id="account-type" label="Tipo">
        <BaseSelect id="account-type" v-model="accountForm.type">
          <option value="bank">Banco</option>
          <option value="cash">Efectivo</option>
          <option value="credit_card">Tarjeta</option>
        </BaseSelect>
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
      <BaseField input-id="opening-balance" label="Saldo inicial">
        <BaseTextInput
          id="opening-balance"
          v-model="accountForm.openingBalance"
          inputmode="decimal"
          placeholder="0"
        />
      </BaseField>
      <BaseButton :loading="props.isCreating" type="submit">
        Crear cuenta
      </BaseButton>
    </form>

    <BaseAlert v-if="props.hasLoadError" class="form-alert" tone="danger">
      No se pudieron cargar las cuentas.
    </BaseAlert>

    <ul class="account-list">
      <li v-for="account in props.accounts" :key="account.id">
        <div>
          <strong>{{ account.displayName }}</strong>
          <span>{{ accountTypeLabel(account.type) }} · {{ account.currency }}</span>
        </div>
        <b>{{ formatMoney(account.currentBalance, account.currency) }}</b>
      </li>
    </ul>
  </BaseCard>
</template>

<style scoped>
.panel-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-block-end: 0;
}

.panel-heading h2 {
  margin: 0;
  color: var(--color-primary-strong);
  font-size: 1rem;
}

.stacked-form {
  display: grid;
  gap: var(--space-3);
  margin-block-end: var(--space-4);
}

.form-alert {
  margin-block-end: var(--space-4);
}

.account-list {
  display: grid;
  gap: var(--space-2);
  list-style: none;
  margin: 0;
  padding: 0;
}

.account-list li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  border-block-start: 1px solid var(--color-border);
  padding-block: var(--space-3);
}

.account-list div {
  display: grid;
  gap: var(--space-1);
  min-inline-size: 0;
}

.account-list span {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}

.account-list strong {
  overflow-wrap: anywhere;
}

.account-list b {
  font-family: var(--font-mono);
  font-size: var(--font-size-sm);
  white-space: nowrap;
}

@media (max-width: 680px) {
  .account-list li {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>
