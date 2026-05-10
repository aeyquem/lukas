import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import BaseSelect from "@/common/components/base/BaseSelect.vue";

describe("BaseSelect", () => {
  it("emits selection changes", async () => {
    const wrapper = mount(BaseSelect, {
      props: {
        modelValue: "bank",
        options: [
          { label: "Banco", value: "bank" },
          { label: "Efectivo", value: "cash" },
        ],
      },
    });

    await wrapper.getComponent({ name: "Select" }).vm.$emit(
      "update:modelValue",
      "cash",
    );

    expect(wrapper.emitted("update:modelValue")).toEqual([["cash"]]);
  });
});
