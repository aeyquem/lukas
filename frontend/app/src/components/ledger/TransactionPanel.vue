<script setup lang="ts">
import { computed, reactive, watch } from "vue";

import type {
  AccountDto,
  AccountEffect,
  CreateManualTransactionRequest,
  TransactionKind,
} from "../../api/ledger";
import BaseButton from "../base/BaseButton.vue";
import BaseCard from "../base/BaseCard.vue";
import BaseField from "../base/BaseField.vue";
import BaseSelect from "../base/BaseSelect.vue";
import BaseTextInput from "../base/BaseTextInput.vue";
import BaseTextarea from "../base/BaseTextarea.vue";

const props = defineProps<{
  accounts: AccountDto[];
  isCreating: boolean;
}>();

const emit = defineEmits<{
  "submit-transaction": [request: CreateManualTransactionRequest];
}>();

const transactionForm = reactive({
  kind: "expense" as TransactionKind,
  accountId: "",
  transactionDate: new Date().toISOString().slice(0, 10),
  amount: "",
  accountEffect: "decrease" as AccountEffect,
  description: "",
  notes: "",
});

const selectedAccount = computed<AccountDto | undefined>(() =>
  props.accounts.find((account) => account.id === transactionForm.accountId),
);

watch(
  () => props.accounts,
  (value) => {
    if (!transactionForm.accountId && value.length > 0) {
      transactionForm.accountId = value[0].id;
    }
  },
  { immediate: true },
);

watch(
  () => transactionForm.kind,
  (kind) => {
    transactionForm.accountEffect =
      kind === "expense" ? "decrease" : "increase";
  },
);

function submitTransaction() {
  if (!selectedAccount.value) {
    return;
  }

  emit("submit-transaction", {
    kind: transactionForm.kind,
    accountId: transactionForm.accountId,
    transactionDate: transactionForm.transactionDate,
    amount: transactionForm.amount.trim(),
    currency: selectedAccount.value.currency,
    accountEffect: transactionForm.accountEffect,
    description: transactionForm.description.trim(),
    notes: transactionForm.notes.trim() || null,
  });

  transactionForm.amount = "";
  transactionForm.description = "";
  transactionForm.notes = "";
}
</script>

<template>
  <BaseCard class="panel">
    <template #header>
      <div class="panel-heading">
        <h2>Movimiento</h2>
      </div>
    </template>

    <form class="stacked-form" @submit.prevent="submitTransaction">
      <BaseField input-id="transaction-account" label="Cuenta">
        <BaseSelect
          id="transaction-account"
          v-model="transactionForm.accountId"
          required
        >
          <option disabled value="">Selecciona una cuenta</option>
          <option
            v-for="account in props.accounts"
            :key="account.id"
            :value="account.id"
          >
            {{ account.displayName }}
          </option>
        </BaseSelect>
      </BaseField>
      <BaseField input-id="transaction-kind" label="Tipo">
        <BaseSelect id="transaction-kind" v-model="transactionForm.kind">
          <option value="expense">Gasto</option>
          <option value="income">Ingreso</option>
          <option value="adjustment">Ajuste</option>
        </BaseSelect>
      </BaseField>
      <BaseField input-id="transaction-date" label="Fecha">
        <BaseTextInput
          id="transaction-date"
          v-model="transactionForm.transactionDate"
          required
          type="date"
        />
      </BaseField>
      <BaseField input-id="transaction-amount" label="Monto">
        <BaseTextInput
          id="transaction-amount"
          v-model="transactionForm.amount"
          required
          inputmode="decimal"
        />
      </BaseField>
      <BaseField input-id="transaction-effect" label="Efecto">
        <BaseSelect
          id="transaction-effect"
          v-model="transactionForm.accountEffect"
        >
          <option value="increase">Aumenta</option>
          <option value="decrease">Disminuye</option>
        </BaseSelect>
      </BaseField>
      <BaseField input-id="transaction-description" label="Descripcion">
        <BaseTextInput
          id="transaction-description"
          v-model="transactionForm.description"
          required
          maxlength="240"
        />
      </BaseField>
      <BaseField input-id="transaction-notes" label="Notas">
        <BaseTextarea
          id="transaction-notes"
          v-model="transactionForm.notes"
          maxlength="1000"
          rows="3"
        />
      </BaseField>
      <BaseButton
        :disabled="props.accounts.length === 0"
        :loading="props.isCreating"
        type="submit"
      >
        Registrar movimiento
      </BaseButton>
    </form>
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
</style>
