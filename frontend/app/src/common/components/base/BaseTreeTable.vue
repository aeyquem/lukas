<script setup lang="ts">
import TreeTable from "primevue/treetable";

defineOptions({
  inheritAttrs: false,
});
</script>

<template>
  <TreeTable class="base-tree-table" v-bind="$attrs">
    <template
      v-for="(_, slotName) in $slots"
      #[slotName]="slotProps"
      :key="slotName"
    >
      <slot :name="slotName" v-bind="slotProps ?? {}" />
    </template>
  </TreeTable>
</template>

<style scoped>
.base-tree-table {
  max-inline-size: 100%;
  min-inline-size: 0;
}

.base-tree-table :deep(.p-treetable-table-container) {
  overflow-x: auto;
}
</style>
