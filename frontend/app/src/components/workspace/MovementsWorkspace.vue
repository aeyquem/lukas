<script setup lang="ts">
import { useQuery } from "@pinia/colada";
import { computed, reactive, ref, watch } from "vue";

import {
  getMovements,
  type AccountDto,
  type AccountEffect,
  type CreateManualTransactionRequest,
  type TransactionKind,
} from "@/api/ledger";
import BaseAlert from "@/components/base/BaseAlert.vue";
import BaseBadge from "@/components/base/BaseBadge.vue";
import BaseButton from "@/components/base/BaseButton.vue";
import BaseCard from "@/components/base/BaseCard.vue";
import BaseCheckbox from "@/components/base/BaseCheckbox.vue";
import BaseField from "@/components/base/BaseField.vue";
import BaseSelect from "@/components/base/BaseSelect.vue";
import BaseTextarea from "@/components/base/BaseTextarea.vue";
import BaseTextInput from "@/components/base/BaseTextInput.vue";
import LedgerMovementsTable from "@/components/ledger/LedgerMovementsTable.vue";

const props = defineProps<{
  accounts: AccountDto[];
  createError?: string;
  createTransaction: (request: CreateManualTransactionRequest) => Promise<void>;
  hasAccountsLoadError: boolean;
  isCreating: boolean;
}>();

type DrawerMode = "closed" | "add";

const showOriginalCurrency = ref(false);
const baseCurrency = "CLP";
const drawerMode = ref<DrawerMode>("closed");
const transactionForm = reactive({
  kind: "expense" as TransactionKind,
  accountId: "",
  transactionDate: new Date().toISOString().slice(0, 10),
  amount: "",
  currency: "CLP",
  accountEffect: "decrease" as AccountEffect,
  description: "",
  notes: "",
});

const movementsQuery = useQuery({
  key: ["movements"],
  query: getMovements,
});

const movements = computed(() => movementsQuery.data.value ?? []);
const movementCount = computed(() => movements.value.length);
const isDrawerOpen = computed(() => drawerMode.value !== "closed");
const hasRegisteredAccounts = computed(() => props.accounts.length > 0);

watch(
  () => props.accounts,
  (accounts) => {
    if (transactionForm.accountId === "" && accounts[0]) {
      transactionForm.accountId = accounts[0].id;
      transactionForm.currency = accounts[0].currency;
    }
  },
  { immediate: true },
);

watch(
  () => transactionForm.accountId,
  (accountId) => {
    const account = props.accounts.find(
      (candidate) => candidate.id === accountId,
    );
    if (account) {
      transactionForm.currency = account.currency;
    }
  },
);

watch(
  () => transactionForm.kind,
  (kind) => {
    if (kind === "income") {
      transactionForm.accountEffect = "increase";
    }

    if (kind === "expense") {
      transactionForm.accountEffect = "decrease";
    }
  },
);

function openAddPanel() {
  resetForm();
  drawerMode.value = "add";
}

function closePanel() {
  drawerMode.value = "closed";
}

async function submitTransaction() {
  const notes = transactionForm.notes.trim();
  const request: CreateManualTransactionRequest = {
    kind: transactionForm.kind,
    accountId: transactionForm.accountId,
    transactionDate: transactionForm.transactionDate,
    amount: transactionForm.amount.trim(),
    currency: transactionForm.currency.trim().toUpperCase(),
    accountEffect: transactionForm.accountEffect,
    description: transactionForm.description.trim(),
  };

  if (notes !== "") {
    request.notes = notes;
  }

  try {
    await props.createTransaction(request);
  } catch {
    return;
  }

  resetForm();
  drawerMode.value = "closed";
}

function resetForm() {
  const firstAccount = props.accounts[0];

  transactionForm.kind = "expense";
  transactionForm.accountId = firstAccount ? firstAccount.id : "";
  transactionForm.transactionDate = new Date().toISOString().slice(0, 10);
  transactionForm.amount = "";
  transactionForm.currency = firstAccount ? firstAccount.currency : "CLP";
  transactionForm.accountEffect = "decrease";
  transactionForm.description = "";
  transactionForm.notes = "";
}
</script>

<template>
  <BaseCard class="movements-panel">
    <template #header>
      <div class="movements-heading">
        <div>
          <h2 id="movements-heading">Movimientos</h2>
          <span>Todos los movimientos del libro mayor</span>
        </div>

        <div class="movements-actions">
          <BaseBadge>{{ movementCount }}</BaseBadge>
          <BaseButton
            :disabled="!hasRegisteredAccounts"
            size="sm"
            type="button"
            @click="openAddPanel"
          >
            Agregar movimiento
          </BaseButton>
          <BaseCheckbox
            v-model:checked="showOriginalCurrency"
            label="Moneda original"
          />
        </div>
      </div>
    </template>

    <section aria-labelledby="movements-heading">
      <p class="currency-note">
        Valores mostrados en
        <strong>{{
          showOriginalCurrency ? "moneda original" : baseCurrency
        }}</strong>
      </p>

      <BaseAlert
        v-if="props.hasAccountsLoadError"
        class="movements-alert"
        tone="danger"
      >
        No se pudieron cargar las cuentas para registrar movimientos.
      </BaseAlert>

      <BaseAlert
        v-else-if="!hasRegisteredAccounts"
        class="movements-alert"
        tone="warning"
      >
        Registra una cuenta antes de agregar movimientos.
      </BaseAlert>

      <BaseAlert
        v-if="movementsQuery.error.value"
        class="movements-alert"
        tone="danger"
      >
        No se pudieron cargar los movimientos.
      </BaseAlert>

      <LedgerMovementsTable
        :movements="movements"
        :show-original-currency="showOriginalCurrency"
      />
    </section>
  </BaseCard>

  <Teleport to="body">
    <div
      v-if="isDrawerOpen"
      class="movement-drawer-backdrop"
      role="presentation"
      @click.self="closePanel"
    >
      <aside
        aria-labelledby="movement-drawer-title"
        aria-modal="true"
        class="movement-drawer"
        role="dialog"
      >
        <header class="movement-drawer__header">
          <div>
            <p class="eyebrow">Movimiento</p>
            <h2 id="movement-drawer-title">Agregar movimiento</h2>
          </div>
          <button type="button" @click="closePanel">Cerrar</button>
        </header>

        <form class="movement-form" @submit.prevent="submitTransaction">
          <BaseAlert v-if="props.createError" tone="danger">
            {{ props.createError }}
          </BaseAlert>

          <BaseField input-id="movement-kind" label="Tipo">
            <BaseSelect id="movement-kind" v-model="transactionForm.kind">
              <option value="expense">Gasto</option>
              <option value="income">Ingreso</option>
              <option value="adjustment">Ajuste</option>
            </BaseSelect>
          </BaseField>

          <BaseField input-id="movement-account" label="Cuenta">
            <BaseSelect
              id="movement-account"
              v-model="transactionForm.accountId"
              required
            >
              <option
                v-for="account in props.accounts"
                :key="account.id"
                :value="account.id"
              >
                {{ account.displayName }}
              </option>
            </BaseSelect>
          </BaseField>

          <BaseField input-id="movement-date" label="Fecha">
            <BaseTextInput
              id="movement-date"
              v-model="transactionForm.transactionDate"
              required
              type="date"
            />
          </BaseField>

          <BaseField input-id="movement-amount" label="Monto">
            <BaseTextInput
              id="movement-amount"
              v-model="transactionForm.amount"
              inputmode="decimal"
              pattern="\d+(\.\d+)?"
              placeholder="0"
              required
            />
          </BaseField>

          <BaseField input-id="movement-currency" label="Moneda">
            <BaseTextInput
              id="movement-currency"
              v-model="transactionForm.currency"
              disabled
              maxlength="3"
              pattern="[A-Z]{3}"
              required
            />
          </BaseField>

          <BaseField input-id="movement-effect" label="Efecto en cuenta">
            <BaseSelect
              id="movement-effect"
              v-model="transactionForm.accountEffect"
              required
            >
              <option value="increase">Aumenta saldo</option>
              <option value="decrease">Disminuye saldo</option>
            </BaseSelect>
          </BaseField>

          <BaseField input-id="movement-description" label="Descripción">
            <BaseTextInput
              id="movement-description"
              v-model="transactionForm.description"
              maxlength="240"
              required
            />
          </BaseField>

          <BaseField input-id="movement-notes" label="Notas">
            <BaseTextarea
              id="movement-notes"
              v-model="transactionForm.notes"
              maxlength="1000"
            />
          </BaseField>

          <div class="movement-form__actions">
            <BaseButton type="button" variant="secondary" @click="closePanel">
              Cancelar
            </BaseButton>
            <BaseButton :loading="props.isCreating" type="submit">
              Crear movimiento
            </BaseButton>
          </div>
        </form>
      </aside>
    </div>
  </Teleport>
</template>

<style scoped>
.movements-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-block-end: 0;
}

.movements-heading h2 {
  margin: 0;
  color: var(--color-primary-strong);
  font-size: 1rem;
}

.movements-heading span,
.currency-note {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
}

.movements-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-3);
}

.currency-note {
  margin-block-end: var(--space-4);
}

.currency-note strong {
  color: var(--color-primary-strong);
}

.movements-alert {
  margin-block-end: var(--space-4);
}

.movement-form {
  display: grid;
  gap: var(--space-3);
}

.movement-form__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding-block-start: var(--space-2);
}

@media (max-width: 680px) {
  .movements-heading {
    align-items: stretch;
    flex-direction: column;
  }

  .movements-actions {
    justify-content: space-between;
  }
}
</style>

<style>
.movement-drawer-backdrop {
  position: fixed;
  z-index: 40;
  inset: 0;
  display: flex;
  justify-content: flex-end;
  background: rgb(24 43 26 / 0.28);
}

.movement-drawer {
  display: grid;
  align-content: start;
  gap: var(--space-5);
  inline-size: min(28rem, 100%);
  block-size: 100dvh;
  overflow-y: auto;
  border-inline-start: 1px solid var(--color-border);
  background: var(--color-card);
  padding: var(--space-5);
  box-shadow: var(--shadow-lg);
}

.movement-drawer__header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: var(--space-4);
}

.movement-drawer__header h2 {
  margin: 0;
  color: var(--color-primary-strong);
  font-size: 1.2rem;
}

.movement-drawer__header button {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-card);
  color: var(--color-primary-strong);
  padding: var(--space-2) var(--space-3);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.movement-drawer .eyebrow {
  margin: 0 0 var(--space-1);
  color: var(--color-muted-foreground);
  font-size: var(--font-size-xs);
  font-weight: 800;
  text-transform: uppercase;
}

@media (max-width: 680px) {
  .movement-drawer {
    inline-size: 100%;
  }
}
</style>
