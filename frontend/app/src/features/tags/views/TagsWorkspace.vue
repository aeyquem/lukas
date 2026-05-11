<script setup lang="ts">
import { useMutation, useQuery, useQueryCache } from "@pinia/colada";
import { computed, reactive, ref, watch } from "vue";

import {
  createTag,
  deleteTag,
  getTags,
  updateTag,
  type TagDto,
} from "@/common/api/ledger";
import BaseAlert from "@/common/components/base/BaseAlert.vue";
import BaseBadge from "@/common/components/base/BaseBadge.vue";
import BaseButton from "@/common/components/base/BaseButton.vue";
import BaseCard from "@/common/components/base/BaseCard.vue";
import BaseField from "@/common/components/base/BaseField.vue";
import BaseTextInput from "@/common/components/base/BaseTextInput.vue";

const defaultTagColor = "#64748B";
const queryCache = useQueryCache();
const tagsQuery = useQuery({
  key: ["tags"],
  query: getTags,
});
const createTagMutation = useMutation({
  mutation: createTag,
  async onSettled() {
    await invalidateTags();
  },
});
const updateTagMutation = useMutation({
  mutation: updateTag,
  async onSettled() {
    await invalidateTags();
    await invalidateTagDependents();
  },
});
const deleteTagMutation = useMutation({
  mutation: deleteTag,
  async onSettled() {
    await invalidateTags();
    await invalidateTagDependents();
  },
});
const tagForm = reactive({
  name: "",
  color: defaultTagColor,
});
type SavedTagValues = {
  color: string;
  name: string;
};

type EditableTag = TagDto;

const editableTags = ref<EditableTag[]>([]);
const savedTags = ref<Record<string, SavedTagValues>>({});
const tags = computed(() => editableTags.value);
const canCreateTag = computed(
  () => tagForm.name.trim() !== "" && isHexColor(tagForm.color),
);

watch(
  () => tagsQuery.data.value,
  (loadedTags) => {
    editableTags.value = (loadedTags ?? []).map((tag) => ({
      ...tag,
      color: normalizeHexColor(tag.color),
    }));
    savedTags.value = Object.fromEntries(
      (loadedTags ?? []).map((tag) => [
        tag.id,
        { name: tag.name, color: normalizeHexColor(tag.color) },
      ]),
    );
  },
  { immediate: true },
);

async function submitTag() {
  if (!canCreateTag.value) {
    return;
  }

  await createTagMutation.mutateAsync({
    name: tagForm.name.trim(),
    color: normalizeHexColor(tagForm.color),
  });
  tagForm.name = "";
  tagForm.color = defaultTagColor;
}

async function saveTag(tag: EditableTag) {
  if (!canSaveTag(tag)) {
    return;
  }

  await updateTagMutation.mutateAsync({
    id: tag.id,
    name: tag.name.trim(),
    color: normalizeHexColor(tag.color),
  });
}

async function removeTag(tag: EditableTag) {
  await deleteTagMutation.mutateAsync(tag.id);
}

async function invalidateTags() {
  await queryCache.invalidateQueries({ key: ["tags"] });
}

async function invalidateTagDependents() {
  await queryCache.invalidateQueries({ key: ["movements"] });
  await queryCache.invalidateQueries({ key: ["tag-report"] });
}

function canSaveTag(tag: EditableTag): boolean {
  return isTagDirty(tag) && tag.name.trim() !== "" && isHexColor(tag.color);
}

function isTagDirty(tag: EditableTag): boolean {
  const savedTag = savedTags.value[tag.id];
  if (!savedTag) {
    return false;
  }

  return (
    tag.name.trim() !== savedTag.name ||
    normalizeHexColor(tag.color) !== savedTag.color
  );
}

function restoreTag(tag: EditableTag) {
  const savedTag = savedTags.value[tag.id];
  if (!savedTag) {
    return;
  }

  tag.name = savedTag.name;
  tag.color = savedTag.color;
}

function isHexColor(value: string): boolean {
  return /^#[0-9a-f]{6}$/iu.test(value);
}

function normalizeHexColor(value: string): string {
  return value.toUpperCase();
}
</script>

<template>
  <BaseCard class="tags-panel">
    <template #header>
      <div class="tags-heading">
        <div>
          <h2 id="tags-heading">Etiquetas</h2>
          <span>Clasificación transversal de movimientos</span>
        </div>

        <BaseBadge>{{ tags.length }}</BaseBadge>
      </div>
    </template>

    <section aria-labelledby="tags-heading">
      <BaseAlert v-if="tagsQuery.error.value" class="tags-alert" tone="danger">
        No se pudieron cargar las etiquetas.
      </BaseAlert>

      <BaseAlert
        v-if="createTagMutation.error.value"
        class="tags-alert"
        tone="danger"
      >
        {{ createTagMutation.error.value.message }}
      </BaseAlert>

      <form class="tag-create-form" @submit.prevent="submitTag">
        <BaseField input-id="tag-name" label="Nombre">
          <BaseTextInput
            id="tag-name"
            v-model="tagForm.name"
            maxlength="60"
            required
          />
        </BaseField>

        <BaseField input-id="tag-color" label="Color">
          <input
            id="tag-color"
            v-model="tagForm.color"
            class="color-picker"
            required
            type="color"
            :style="{ '--selected-color': tagForm.color }"
          />
        </BaseField>

        <span class="tag-preview" :style="{ '--tag-color': tagForm.color }">
          {{ tagForm.name.trim() || "Nueva etiqueta" }}
        </span>

        <BaseButton
          :disabled="!canCreateTag"
          :loading="createTagMutation.isLoading.value"
          type="submit"
        >
          Crear etiqueta
        </BaseButton>
      </form>

      <div class="tags-table-wrap">
        <table class="tags-table">
          <thead>
            <tr>
              <th scope="col">Etiqueta</th>
              <th scope="col">Color</th>
              <th scope="col">Vista</th>
              <th scope="col">Acciones</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="tag in tags" :key="tag.id">
              <td>
                <BaseTextInput
                  v-model="tag.name"
                  maxlength="60"
                  :aria-label="`Nombre de etiqueta ${tag.name}`"
                />
              </td>
              <td>
                <input
                  v-model="tag.color"
                  class="color-picker"
                  type="color"
                  :aria-label="`Color de etiqueta ${tag.name}`"
                  :style="{ '--selected-color': tag.color }"
                />
              </td>
              <td>
                <span class="tag-preview" :style="{ '--tag-color': tag.color }">
                  {{ tag.name }}
                </span>
              </td>
              <td>
                <div class="tag-actions">
                  <BaseButton
                    :disabled="!canSaveTag(tag)"
                    size="sm"
                    type="button"
                    @click="saveTag(tag)"
                  >
                    Guardar
                  </BaseButton>
                  <BaseButton
                    :disabled="!isTagDirty(tag)"
                    size="sm"
                    type="button"
                    variant="secondary"
                    @click="restoreTag(tag)"
                  >
                    Deshacer
                  </BaseButton>
                  <BaseButton
                    size="sm"
                    type="button"
                    variant="danger"
                    @click="removeTag(tag)"
                  >
                    Eliminar
                  </BaseButton>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <p v-if="tags.length === 0" class="tags-empty">
        Sin etiquetas registradas.
      </p>
    </section>
  </BaseCard>
</template>

<style scoped>
.tags-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--lu-space-3);
  margin-block-end: 0;
}

.tags-heading h2 {
  margin: 0;
  font-size: var(--lu-font-size-md);
}

.tags-heading span,
.tags-empty {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-sm);
}

.tags-alert {
  margin-block-end: var(--lu-space-4);
}

.tag-create-form {
  display: grid;
  grid-template-columns:
    minmax(14rem, 1fr) minmax(5rem, 7rem) minmax(9rem, max-content)
    max-content;
  gap: var(--lu-space-3);
  align-items: end;
  margin-block-end: var(--lu-space-5);
}

.tags-table-wrap {
  overflow-x: auto;
}

.tags-table {
  inline-size: 100%;
  min-inline-size: 42rem;
  border-collapse: collapse;
}

.tags-table th,
.tags-table td {
  border-block-end: 1px solid var(--lu-color-border);
  padding: var(--lu-space-3);
  text-align: start;
  vertical-align: middle;
}

.tags-table th {
  color: var(--lu-color-text-muted);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
  text-transform: uppercase;
}

.tag-preview {
  --tag-color: #64748b;
  display: inline-flex;
  align-items: center;
  min-block-size: 1.65rem;
  border: 1px solid color-mix(in srgb, var(--tag-color) 42%, transparent);
  border-radius: var(--lu-radius-pill);
  background: color-mix(in srgb, var(--tag-color) 14%, transparent);
  color: var(--lu-color-text);
  padding-inline: var(--lu-space-3);
  font-size: var(--lu-font-size-xs);
  font-weight: 800;
}

.color-picker {
  inline-size: 100%;
  min-inline-size: 4.5rem;
  block-size: var(--lu-size-control-sm);
  border: 1px solid color-mix(in srgb, var(--selected-color) 52%, var(--lu-color-border));
  border-radius: var(--lu-radius-md);
  background: var(--selected-color);
  cursor: pointer;
  padding: 0.15rem;
}

.color-picker::-webkit-color-swatch-wrapper {
  padding: 0;
}

.color-picker::-webkit-color-swatch {
  border: 0;
  border-radius: calc(var(--lu-radius-md) - 2px);
}

.color-picker::-moz-color-swatch {
  border: 0;
  border-radius: calc(var(--lu-radius-md) - 2px);
}

.tag-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--lu-space-2);
}

@media (max-width: 680px) {
  .tags-heading,
  .tag-create-form {
    align-items: stretch;
    grid-template-columns: 1fr;
  }

  .tags-heading {
    flex-direction: column;
  }
}
</style>
