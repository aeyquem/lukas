<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { FilterOperator, FilterMatchMode } from "@primevue/core/api";
import type { DataTableFilterMeta } from "primevue/datatable";

import BaseColumn from "@/components/base/BaseColumn.vue";
import BaseDataTable from "@/components/base/BaseDataTable.vue";
import BaseMultiSelect from "@/components/base/BaseMultiSelect.vue";
import BaseSelect from "@/components/base/BaseSelect.vue";
import BaseTextInput from "@/components/base/BaseTextInput.vue";
import { formatDate, formatMoney } from "@/formatters";
import type { LedgerMovement, MovementDirection } from "@/ledger/movements";

type DisplayMovement = LedgerMovement & {
  displayAmount: string;
  displayCurrency: string;
  categoryDisplay: string;
  isUncategorized: boolean;
  signedDisplayAmount: string;
  displayAmountNumber: string;
  displayAmountPrefix: string;
};

type GroupField = "none" | "accountName";

type GroupOption = {
  label: string;
  value: GroupField;
};

type AccountOption = {
  label: string;
  value: string;
};

const props = withDefaults(
  defineProps<{
    movements: LedgerMovement[];
    showOriginalCurrency?: boolean;
    groupBy?: GroupField;
  }>(),
  {
    groupBy: "none",
    showOriginalCurrency: false,
  },
);

const emit = defineEmits<{
  editMovement: [movement: LedgerMovement];
}>();

const selectedAccountNames = ref<string[]>([]);
const selectedGroupBy = ref<GroupField>(props.groupBy);
const filters = ref<DataTableFilterMeta>({
  notes: {
    operator: FilterOperator.AND,
    constraints: [{ value: "", matchMode: FilterMatchMode.CONTAINS }],
  },
});

const groupOptions: GroupOption[] = [
  { label: "Sin agrupar", value: "none" },
  { label: "Cuenta", value: "accountName" },
];

const accountOptions = computed<AccountOption[]>(() =>
  Array.from(new Set(props.movements.map((movement) => movement.accountName)))
    .sort((firstAccount, secondAccount) =>
      firstAccount.localeCompare(secondAccount, "es-CL"),
    )
    .map((accountName) => ({
      label: accountName,
      value: accountName,
    })),
);

watch(
  accountOptions,
  (options) => {
    const accountNames = options.map((account) => account.value);
    const stillAvailable = selectedAccountNames.value.filter((accountName) =>
      accountNames.includes(accountName),
    );
    const newlyAvailable = accountNames.filter(
      (accountName) => !stillAvailable.includes(accountName),
    );

    selectedAccountNames.value = [...stillAvailable, ...newlyAvailable];
  },
  { immediate: true },
);

const selectedAccounts = computed<string[]>(() => {
  return selectedAccountNames.value;
});

const activeGroupBy = computed(() =>
  selectedGroupBy.value === "none" ? undefined : selectedGroupBy.value,
);

const sortField = computed(() => activeGroupBy.value ?? "date");
const sortOrder = computed(() => (activeGroupBy.value ? 1 : -1));

const displayMovements = computed<DisplayMovement[]>(() =>
  props.movements
    .map((movement) => {
      const displayAmount = props.showOriginalCurrency
        ? movement.originalAmount
        : movement.baseAmount;
      const displayCurrency = props.showOriginalCurrency
        ? movement.originalCurrency
        : movement.baseCurrency;
      const amountParts = signedAmountParts(
        movement.direction,
        displayAmount,
        displayCurrency,
      );

      return {
        ...movement,
        displayAmount,
        displayCurrency,
        categoryDisplay:
          movement.envelopeName ?? movement.categoryName ?? "Sin categoría",
        isUncategorized: !movement.envelopeName && !movement.categoryName,
        signedDisplayAmount: `${amountParts.prefix}${amountParts.number}`,
        displayAmountNumber: amountParts.number,
        displayAmountPrefix: amountParts.prefix,
      };
    })
    .filter((movement) =>
      selectedAccounts.value.includes(movement.accountName),
    ),
);

function signedAmountParts(
  direction: MovementDirection,
  amount: string,
  currency: string,
): { number: string; prefix: string } {
  const sign = direction === "out" ? "-" : "+";
  const formattedAmount = formatMoney(amount, currency);
  const amountMatch = /^(\D*)(.*)$/.exec(formattedAmount);

  if (!amountMatch) {
    return { number: formattedAmount, prefix: sign };
  }

  return {
    number: amountMatch[2],
    prefix: `${sign}${amountMatch[1]}`,
  };
}

function groupHeading(data: DisplayMovement): string {
  if (!activeGroupBy.value) {
    return "";
  }

  return String(data[activeGroupBy.value]);
}

function movementRowClass(data: DisplayMovement): string {
  return data.isUncategorized ? "movement-row--uncategorized" : "";
}

function editMovement(movement: DisplayMovement) {
  emit("editMovement", movement);
}
</script>

<template>
  <BaseDataTable
    v-model:filters="filters"
    class="ledger-movements-table"
    column-resize-mode="fit"
    data-key="id"
    :group-rows-by="activeGroupBy"
    paginator
    removable-sort
    reorderable-columns
    resizable-columns
    filter-display="menu"
    :row-class="movementRowClass"
    :row-group-mode="activeGroupBy ? 'subheader' : undefined"
    :rows="20"
    show-gridlines
    :sort-field="sortField"
    :sort-order="sortOrder"
    :value="displayMovements"
  >
    <template #header>
      <div class="table-toolbar">
        <div class="table-filter">
          <label for="movement-table-accounts">Cuentas</label>
          <BaseMultiSelect
            id="movement-table-accounts"
            v-model="selectedAccountNames"
            display="chip"
            :max-selected-labels="accountOptions.length - 1"
            filter
            :options="accountOptions"
            placeholder="Seleccionar"
            selected-items-label="Todas las cuentas"
          />
        </div>

        <div class="table-filter table-filter--group">
          <label for="movement-table-group">Agrupar</label>
          <BaseSelect
            id="movement-table-group"
            v-model="selectedGroupBy"
            :options="groupOptions"
          />
        </div>
      </div>
    </template>

    <template #groupheader="{ data }">
      <span class="group-heading">{{ groupHeading(data) }}</span>
    </template>

    <BaseColumn field="date" header="Fecha" sortable style="width: 9.5rem">
      <template #body="{ data }">
        {{ formatDate(data.date) }}
      </template>
    </BaseColumn>

    <BaseColumn
      field="accountName"
      header="Cuenta"
      sortable
      style="width: 14rem"
    >
      <template #body="{ data }">
        <strong>{{ data.accountName }}</strong>
      </template>
    </BaseColumn>

    <BaseColumn
      field="description"
      header="Detalle"
      sortable
      style="min-width: 16rem"
    />

    <BaseColumn
      field="categoryDisplay"
      header="Categoría"
      sortable
      style="min-width: 12rem"
    >
      <template #body="{ data }">
        <span
          class="category-state"
          :data-state="data.isUncategorized ? 'missing' : 'set'"
        >
          <span v-if="data.isUncategorized" aria-hidden="true">!</span>
          {{ data.categoryDisplay }}
        </span>
      </template>
    </BaseColumn>

    <BaseColumn
      field="notes"
      header="Notas"
      style="min-width: 14rem"
      filter-field="notes"
      :filter-match-mode="FilterMatchMode.CONTAINS"
      :show-add-button="false"
      :show-filter-match-modes="false"
      :show-filter-operator="false"
    >
      <template #body="{ data }">
        <span class="notes">{{ data.notes || "Sin notas" }}</span>
      </template>
      <template #filter="{ filterModel }">
        <BaseTextInput
          v-model="filterModel.value"
          class="notes-filter"
          placeholder="Buscar notas"
          type="text"
        />
      </template>
    </BaseColumn>

    <BaseColumn
      field="displayCurrency"
      header="Moneda"
      sortable
      style="width: 8rem"
    />

    <BaseColumn
      field="signedDisplayAmount"
      header="Valor"
      sortable
      style="width: 12rem"
    >
      <template #body="{ data }">
        <span
          class="numeric"
          :aria-label="data.signedDisplayAmount"
          :data-direction="data.direction"
        >
          <span class="numeric__prefix">{{ data.displayAmountPrefix }}</span>
          <span class="numeric__amount">{{ data.displayAmountNumber }}</span>
        </span>
      </template>
    </BaseColumn>

    <BaseColumn header="Acciones" style="width: 7rem">
      <template #body="{ data }">
        <button class="row-action" type="button" @click="editMovement(data)">
          Editar
        </button>
      </template>
    </BaseColumn>

    <template #empty>Sin movimientos para mostrar.</template>
  </BaseDataTable>
</template>

<style scoped>
.table-toolbar {
  display: grid;
  grid-template-columns: minmax(14rem, 1.2fr) minmax(11rem, 0.65fr);
  gap: var(--space-2);
}

.table-filter {
  display: grid;
  align-content: start;
  gap: var(--space-1);
}

.table-filter label {
  color: var(--color-muted-foreground);
  font-size: var(--font-size-sm);
  font-weight: 700;
}

.table-filter :deep(.p-multiselect),
.table-filter :deep(.p-select) {
  inline-size: 100%;
}

.group-heading {
  color: var(--color-primary-strong);
  font-size: var(--font-size-sm);
  font-weight: 800;
}

.numeric {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: var(--space-1);
  font-family: var(--font-mono);
  font-weight: 700;
  inline-size: 100%;
  white-space: nowrap;
}

.numeric__prefix {
  justify-self: start;
}

.numeric__amount {
  justify-self: end;
}

.notes {
  color: var(--color-muted-foreground);
  overflow-wrap: anywhere;
}

.category-state {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-block-size: 1.65rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-round);
  padding-inline: var(--space-3);
  font-size: var(--font-size-xs);
  font-weight: 800;
}

.category-state[data-state="missing"] {
  border-color: color-mix(in srgb, var(--color-warning) 36%, transparent);
  background: color-mix(in srgb, var(--color-warning) 12%, transparent);
  color: var(--color-warning);
}

.category-state[data-state="set"] {
  border-color: color-mix(in srgb, var(--color-primary) 28%, transparent);
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
  color: var(--color-primary-strong);
}

.notes-filter {
  inline-size: 100%;
}

.numeric[data-direction="in"] {
  color: var(--color-primary-strong);
}

.numeric[data-direction="out"] {
  color: var(--color-danger-strong);
}

.row-action {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-card);
  color: var(--color-primary-strong);
  padding: var(--space-1) var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 800;
}

.row-action:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

:deep(.p-datatable-header) {
  border-start-start-radius: var(--radius-lg);
  border-start-end-radius: var(--radius-lg);
}

:deep(.p-datatable-table) {
  font-size: var(--font-size-sm);
}

:deep(.p-datatable-column-title),
:deep(.p-paginator) {
  font-size: var(--font-size-sm);
}

:deep(.p-datatable-thead > tr > th) {
  white-space: nowrap;
}

:deep(.movement-row--uncategorized > td) {
  background: color-mix(in srgb, var(--color-warning) 7%, transparent);
}

@media (max-width: 680px) {
  .table-toolbar {
    grid-template-columns: 1fr;
  }
}
</style>
