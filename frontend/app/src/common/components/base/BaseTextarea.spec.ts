import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import BaseTextarea from "@/common/components/base/BaseTextarea.vue";

describe("BaseTextarea", () => {
  it("emits value updates", async () => {
    const wrapper = mount(BaseTextarea, {
      props: {
        modelValue: "",
      },
    });

    await wrapper.get("textarea").setValue("Nota interna");

    expect(wrapper.emitted("update:modelValue")).toEqual([["Nota interna"]]);
  });
});
