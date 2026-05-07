import { useMutation, useQuery, useQueryCache } from "@pinia/colada";
import { computed, ref } from "vue";

import {
  createAccount,
  getAccounts,
  type CreateAccountRequest,
} from "../api/ledger";
import { currentPeriod } from "../formatters";

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

  const accounts = computed(() => accountsQuery.data.value ?? []);

  async function invalidateAccounts() {
    await queryCache.invalidateQueries({ key: ["accounts"] });
  }

  function submitAccount(request: CreateAccountRequest) {
    accountMutation.mutate(request);
  }

  return {
    selectedPeriod,
    accountsQuery,
    accountMutation,
    accounts,
    submitAccount,
  };
}
