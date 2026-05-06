import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import BaseSelect from "./BaseSelect.vue";

describe("BaseSelect", () => {
  it("emits selection changes", async () => {
    const wrapper = mount(BaseSelect, {
      props: {
        modelValue: "bank",
      },
      slots: {
        default:
          '<option value="bank">Banco</option><option value="cash">Efectivo</option>',
      },
    });

    await wrapper.get("select").setValue("cash");

    expect(wrapper.emitted("update:modelValue")).toEqual([["cash"]]);
  });
});
