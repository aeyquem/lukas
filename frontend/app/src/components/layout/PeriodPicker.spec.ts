import { mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import PeriodPicker from "@/components/layout/PeriodPicker.vue";

const baseButtonStub = {
  name: "BaseButton",
  props: {
    disabled: Boolean,
    size: String,
    type: String,
    variant: String,
  },
  emits: ["click"],
  template: `
    <button :disabled="disabled" :type="type" @click="$emit('click', $event)">
      <slot />
    </button>
  `,
};

const baseDatePickerStub = {
  name: "BaseDatePicker",
  props: {
    dateFormat: String,
    inputId: String,
    manualInput: Boolean,
    modelValue: Date,
    view: String,
  },
  emits: ["update:modelValue"],
  template: `
    <button
      class="date-picker"
      type="button"
      :aria-label="$attrs['aria-label']"
      @click="$emit('update:modelValue', new Date(2026, 6, 1))"
    >
      {{ $attrs['aria-label'] }}
    </button>
  `,
};

function mountPeriodPicker(period: string) {
  return mount(PeriodPicker, {
    global: {
      stubs: {
        BaseButton: baseButtonStub,
        BaseDatePicker: baseDatePickerStub,
      },
    },
    props: {
      inputId: "period",
      modelValue: period,
    },
  });
}

describe("PeriodPicker", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 4, 10));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows the selected month and year through the date picker", () => {
    const wrapper = mountPeriodPicker("2026-05");
    const datePicker = wrapper.getComponent({ name: "BaseDatePicker" });

    expect(wrapper.get(".date-picker").text()).toBe("mayo de 2026");
    expect(datePicker.props("dateFormat")).toBe("MM yy");
    expect(datePicker.props("inputId")).toBe("period");
    expect(datePicker.props("manualInput")).toBe(false);
    expect(datePicker.props("view")).toBe("month");
  });

  it("emits the selected month from the date picker", async () => {
    const wrapper = mountPeriodPicker("2026-05");

    await wrapper.get(".date-picker").trigger("click");

    expect(wrapper.emitted("update:modelValue")).toEqual([["2026-07"]]);
  });

  it("moves the period one month at a time", async () => {
    const wrapper = mountPeriodPicker("2026-01");

    await wrapper.get('[aria-label="Periodo anterior"]').trigger("click");
    await wrapper.setProps({ modelValue: "2026-01" });
    await wrapper.get('[aria-label="Periodo siguiente"]').trigger("click");

    expect(wrapper.emitted("update:modelValue")).toEqual([
      ["2025-12"],
      ["2026-02"],
    ]);
  });

  it("moves to the current period from the today button", async () => {
    const wrapper = mountPeriodPicker("2026-03");

    await wrapper.get('[aria-label="Ir al periodo actual"]').trigger("click");

    expect(wrapper.emitted("update:modelValue")).toEqual([["2026-05"]]);
  });

  it("disables the today button in the current period", async () => {
    const wrapper = mountPeriodPicker("2026-05");
    const todayButton = wrapper.get('[aria-label="Ir al periodo actual"]');

    expect(todayButton.attributes("disabled")).toBeDefined();

    await todayButton.trigger("click");

    expect(wrapper.emitted("update:modelValue")).toBeUndefined();
  });
});
