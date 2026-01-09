import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { FormProvider, useForm } from "react-hook-form";
import { ProviderPresetSelector } from "@/components/providers/forms/ProviderPresetSelector";
import type { ProviderPreset } from "@/config/claudeProviderPresets";

const tMock = vi.fn((key: string, options?: Record<string, unknown>) => {
  if (options?.defaultValue) {
    return String(options.defaultValue);
  }
  return key;
});

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: tMock }),
}));

function TestWrapper({ children }: { children: React.ReactNode }) {
  const form = useForm({ defaultValues: {} });
  return <FormProvider {...form}>{children}</FormProvider>;
}

const createPreset = (
  overrides: Partial<ProviderPreset> = {}
): ProviderPreset => ({
  name: overrides.name ?? "Test Preset",
  category: overrides.category ?? "third_party",
  websiteUrl: overrides.websiteUrl ?? "https://test.com",
  settingsConfig: overrides.settingsConfig ?? { env: {} },
  isPartner: overrides.isPartner ?? false,
  theme: overrides.theme,
});

const defaultProps = {
  selectedPresetId: null,
  groupedPresets: {
    third_party: [
      { id: "preset-1", preset: createPreset({ name: "Preset 1" }) },
      { id: "preset-2", preset: createPreset({ name: "Preset 2" }) },
    ],
  },
  categoryKeys: ["third_party"],
  presetCategoryLabels: { third_party: "Third Party" },
  onPresetChange: vi.fn(),
};

beforeEach(() => {
  tMock.mockClear();
  defaultProps.onPresetChange.mockClear();
});

describe("ProviderPresetSelector", () => {
  it("renders custom button", () => {
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} />
      </TestWrapper>
    );

    expect(
      screen.getByRole("button", { name: "providerPreset.custom" })
    ).toBeInTheDocument();
  });

  it("renders preset buttons from groupedPresets", () => {
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} />
      </TestWrapper>
    );

    expect(screen.getByRole("button", { name: "Preset 1" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Preset 2" })).toBeInTheDocument();
  });

  it("calls onPresetChange when custom button clicked", async () => {
    const user = userEvent.setup();
    const onPresetChange = vi.fn();
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} onPresetChange={onPresetChange} />
      </TestWrapper>
    );

    await user.click(screen.getByRole("button", { name: "providerPreset.custom" }));

    expect(onPresetChange).toHaveBeenCalledWith("custom");
  });

  it("calls onPresetChange when preset button clicked", async () => {
    const user = userEvent.setup();
    const onPresetChange = vi.fn();
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} onPresetChange={onPresetChange} />
      </TestWrapper>
    );

    await user.click(screen.getByRole("button", { name: "Preset 1" }));

    expect(onPresetChange).toHaveBeenCalledWith("preset-1");
  });

  it("shows selected state for custom button", () => {
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} selectedPresetId="custom" />
      </TestWrapper>
    );

    const customBtn = screen.getByRole("button", { name: "providerPreset.custom" });
    expect(customBtn).toHaveClass("bg-blue-500");
  });

  it("shows selected state for preset button", () => {
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} selectedPresetId="preset-1" />
      </TestWrapper>
    );

    const presetBtn = screen.getByRole("button", { name: "Preset 1" });
    expect(presetBtn).toHaveClass("bg-blue-500");
  });

  it("shows hint text for official category", () => {
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} category="official" />
      </TestWrapper>
    );

    expect(
      screen.getByText("💡 官方供应商使用浏览器登录，无需配置 API Key")
    ).toBeInTheDocument();
  });

  it("shows hint text for third_party category", () => {
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} category="third_party" />
      </TestWrapper>
    );

    expect(
      screen.getByText("💡 第三方供应商需要填写 API Key 和请求地址")
    ).toBeInTheDocument();
  });

  it("shows hint text for custom category", () => {
    render(
      <TestWrapper>
        <ProviderPresetSelector {...defaultProps} category="custom" />
      </TestWrapper>
    );

    expect(
      screen.getByText("💡 自定义配置需手动填写所有必要字段")
    ).toBeInTheDocument();
  });

  it("shows partner star badge for partner presets", () => {
    const partnerPresets = {
      third_party: [
        {
          id: "partner-1",
          preset: createPreset({ name: "Partner Preset", isPartner: true }),
        },
      ],
    };

    render(
      <TestWrapper>
        <ProviderPresetSelector
          {...defaultProps}
          groupedPresets={partnerPresets}
        />
      </TestWrapper>
    );

    const partnerBtn = screen.getByRole("button", { name: "Partner Preset" });
    expect(partnerBtn.querySelector("svg")).toBeInTheDocument();
  });

  it("renders preset with custom theme icon", () => {
    const themedPresets = {
      third_party: [
        {
          id: "themed-1",
          preset: createPreset({
            name: "Claude Preset",
            theme: { icon: "claude" as const },
          }),
        },
      ],
    };

    render(
      <TestWrapper>
        <ProviderPresetSelector
          {...defaultProps}
          groupedPresets={themedPresets}
        />
      </TestWrapper>
    );

    expect(
      screen.getByRole("button", { name: "Claude Preset" })
    ).toBeInTheDocument();
  });

  it("applies custom background color when selected", () => {
    const themedPresets = {
      third_party: [
        {
          id: "themed-1",
          preset: createPreset({
            name: "Themed Preset",
            theme: { backgroundColor: "#FF0000", textColor: "#FFFFFF" },
          }),
        },
      ],
    };

    render(
      <TestWrapper>
        <ProviderPresetSelector
          {...defaultProps}
          groupedPresets={themedPresets}
          selectedPresetId="themed-1"
        />
      </TestWrapper>
    );

    const btn = screen.getByRole("button", { name: "Themed Preset" });
    expect(btn).toHaveStyle({ backgroundColor: "#FF0000" });
  });
});
