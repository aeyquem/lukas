import { useMutation, useQuery, useQueryCache } from "@pinia/colada";
import { computed, ref } from "vue";

import {
  createAccount,
  createManualTransaction,
  getAccounts,
  updateManualTransaction,
  type CreateAccountRequest,
  type CreateManualTransactionRequest,
  type UpdateManualTransactionRequest,
} from "@/api/ledger";
import { currentPeriod } from "@/formatters";

export function useLedgerWorkspace() {
  const queryCache = useQueryCache();
  const selectedPeriod = ref(currentPeriod());

  const accountsQuery = useQuery({
    key: ["accounts"],
    query: getAccounts,
  });

  const accountMutation = useMutation({
    mutation: createAccount,
    async onSettled() {
      await invalidateAccounts();
    },
  });

  const transactionMutation = useMutation({
    mutation: createManualTransaction,
    async onSettled() {
      await invalidateAccounts();
      await invalidateMovements();
      await invalidateBudget();
    },
  });

  const updateTransactionMutation = useMutation({
    mutation: updateManualTransaction,
    async onSettled() {
      await invalidateAccounts();
      await invalidateMovements();
      await invalidateBudget();
    },
  });

  const accounts = computed(() => accountsQuery.data.value ?? []);

  async function invalidateAccounts() {
    await queryCache.invalidateQueries({ key: ["accounts"] });
  }

  async function invalidateMovements() {
    await queryCache.invalidateQueries({ key: ["movements"] });
  }

  async function invalidateBudget() {
    await queryCache.invalidateQueries({ key: ["budget-tree"] });
  }

  function submitAccount(request: CreateAccountRequest) {
    accountMutation.mutate(request);
  }

  async function submitTransaction(request: CreateManualTransactionRequest) {
    await transactionMutation.mutateAsync(request);
  }

  async function updateTransaction(request: UpdateManualTransactionRequest) {
    await updateTransactionMutation.mutateAsync(request);
  }

  return {
    selectedPeriod,
    accountsQuery,
    accountMutation,
    transactionMutation,
    updateTransactionMutation,
    accounts,
    submitAccount,
    submitTransaction,
    updateTransaction,
  };
}
