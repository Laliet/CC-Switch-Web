import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ProviderForm } from "@/components/providers/forms/ProviderForm";

const tMock = vi.fn((key: string, options?: Record<string, unknown>) => {
  if (options?.defaultValue) {
    return String(options.defaultValue);
  }
  return key;
});

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: tMock }),
}));

// Mock all child components to simplify testing
vi.mock("@/components/providers/forms/ProviderPresetSelector", () => ({
  ProviderPresetSelector: ({ onPresetChange, selectedPresetId }: any) => (
    <div data-testid="preset-selector">
      <button onClick={() => onPresetChange("custom")}>Select Custom</button>
      <button onClick={() => onPresetChange("claude-0")}>Select Preset</button>
      <span data-testid="selected-preset">{selectedPresetId}</span>
    </div>
  ),
}));

vi.mock("@/components/providers/forms/BasicFormFields", () => ({
  BasicFormFields: ({ form }: any) => (
    <div data-testid="basic-fields">
      <input
        data-testid="name-input"
        value={form.watch("name")}
        onChange={(e) => form.setValue("name", e.target.value)}
        placeholder="Name"
      />
      <input
        data-testid="website-input"
        value={form.watch("websiteUrl")}
        onChange={(e) => form.setValue("websiteUrl", e.target.value)}
        placeholder="Website"
      />
    </div>
  ),
}));

vi.mock("@/components/providers/forms/ClaudeFormFields", () => ({
  ClaudeFormFields: () => <div data-testid="claude-fields">Claude Fields</div>,
}));

vi.mock("@/components/providers/forms/CodexFormFields", () => ({
  CodexFormFields: () => <div data-testid="codex-fields">Codex Fields</div>,
}));

vi.mock("@/components/providers/forms/GeminiFormFields", () => ({
  GeminiFormFields: () => <div data-testid="gemini-fields">Gemini Fields</div>,
}));

vi.mock("@/components/providers/forms/CommonConfigEditor", () => ({
  CommonConfigEditor: ({ value, onChange }: any) => (
    <div data-testid="common-config-editor">
      <textarea
        data-testid="config-textarea"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  ),
}));

vi.mock("@/components/providers/forms/CodexConfigEditor", () => ({
  default: () => <div data-testid="codex-config-editor">Codex Config</div>,
}));

vi.mock("@/components/providers/forms/GeminiConfigEditor", () => ({
  default: () => <div data-testid="gemini-config-editor">Gemini Config</div>,
}));

// Mock hooks
vi.mock("@/components/providers/forms/hooks", () => ({
  useProviderCategory: () => ({ category: "third_party" }),
  useApiKeyState: () => ({
    apiKey: "",
    handleApiKeyChange: vi.fn(),
    showApiKey: () => true,
  }),
  useBaseUrlState: () => ({
    baseUrl: "",
    handleClaudeBaseUrlChange: vi.fn(),
  }),
  useModelState: () => ({
    claudeModel: "",
    defaultHaikuModel: "",
    defaultSonnetModel: "",
    defaultOpusModel: "",
    handleModelChange: vi.fn(),
  }),
  useCodexConfigState: () => ({
    codexAuth: "{}",
    codexConfig: "",
    codexApiKey: "",
    codexBaseUrl: "",
    codexModelName: "",
    codexAuthError: null,
    setCodexAuth: vi.fn(),
    handleCodexApiKeyChange: vi.fn(),
    handleCodexBaseUrlChange: vi.fn(),
    handleCodexModelNameChange: vi.fn(),
    handleCodexConfigChange: vi.fn(),
    resetCodexConfig: vi.fn(),
  }),
  useCodexTomlValidation: () => ({
    configError: null,
    debouncedValidate: vi.fn(),
  }),
  useTemplateValues: () => ({
    templateValues: {},
    templateValueEntries: [],
    selectedPreset: null,
    handleTemplateValueChange: vi.fn(),
    validateTemplateValues: () => ({ isValid: true }),
  }),
  useCommonConfigSnippet: () => ({
    useCommonConfig: false,
    commonConfigSnippet: "",
    commonConfigError: null,
    handleCommonConfigToggle: vi.fn(),
    handleCommonConfigSnippetChange: vi.fn(),
  }),
  useCodexCommonConfig: () => ({
    useCommonConfig: false,
    commonConfigSnippet: "",
    commonConfigError: null,
    handleCommonConfigToggle: vi.fn(),
    handleCommonConfigSnippetChange: vi.fn(),
  }),
  useApiKeyLink: () => ({
    shouldShowApiKeyLink: false,
    websiteUrl: "",
    isPartner: false,
    partnerPromotionKey: undefined,
  }),
  useSpeedTestEndpoints: () => [],
  useGeminiConfigState: () => ({
    geminiEnv: "",
    geminiConfig: "",
    geminiApiKey: "",
    geminiBaseUrl: "",
    geminiModel: "",
    envError: null,
    configError: null,
    handleGeminiApiKeyChange: vi.fn(),
    handleGeminiBaseUrlChange: vi.fn(),
    handleGeminiEnvChange: vi.fn(),
    handleGeminiConfigChange: vi.fn(),
    resetGeminiConfig: vi.fn(),
    envStringToObj: () => ({}),
    envObjToString: () => "",
  }),
  useGeminiCommonConfig: () => ({
    useCommonConfig: false,
    commonConfigSnippet: "",
    commonConfigError: null,
    handleCommonConfigToggle: vi.fn(),
    handleCommonConfigSnippetChange: vi.fn(),
  }),
}));

// Mock presets
vi.mock("@/config/claudeProviderPresets", () => ({
  providerPresets: [
    {
      name: "Test Preset",
      category: "third_party",
      websiteUrl: "https://test.com",
      settingsConfig: { env: {} },
    },
  ],
}));

vi.mock("@/config/codexProviderPresets", () => ({
  codexProviderPresets: [
    {
      name: "Codex Preset",
      category: "third_party",
      websiteUrl: "https://codex.com",
      auth: {},
      config: "",
    },
  ],
}));

vi.mock("@/config/geminiProviderPresets", () => ({
  geminiProviderPresets: [
    {
      name: "Gemini Preset",
      category: "official",
      websiteUrl: "https://gemini.com",
      settingsConfig: { env: {} },
    },
  ],
}));

vi.mock("@/config/codexTemplates", () => ({
  getCodexCustomTemplate: () => ({ auth: {}, config: "" }),
}));

vi.mock("@/utils/providerConfigUtils", () => ({
  applyTemplateValues: (config: any) => config,
}));

vi.mock("@/utils/providerMetaUtils", () => ({
  mergeProviderMeta: (existing: any, custom: any) => ({ ...existing, ...custom }),
}));

const defaultProps = {
  appId: "claude" as const,
  submitLabel: "Save",
  onSubmit: vi.fn(),
  onCancel: vi.fn(),
};

beforeEach(() => {
  tMock.mockClear();
  defaultProps.onSubmit.mockClear();
  defaultProps.onCancel.mockClear();
});

describe("ProviderForm", () => {
  it("renders form with basic fields", () => {
    render(<ProviderForm {...defaultProps} />);

    expect(screen.getByTestId("basic-fields")).toBeInTheDocument();
  });

  it("renders preset selector in create mode", () => {
    render(<ProviderForm {...defaultProps} />);

    expect(screen.getByTestId("preset-selector")).toBeInTheDocument();
  });

  it("hides preset selector in edit mode", () => {
    render(
      <ProviderForm
        {...defaultProps}
        initialData={{ name: "Existing Provider" }}
      />
    );

    expect(screen.queryByTestId("preset-selector")).not.toBeInTheDocument();
  });

  it("renders Claude fields for claude appId", () => {
    render(<ProviderForm {...defaultProps} appId="claude" />);

    expect(screen.getByTestId("claude-fields")).toBeInTheDocument();
    expect(screen.queryByTestId("codex-fields")).not.toBeInTheDocument();
    expect(screen.queryByTestId("gemini-fields")).not.toBeInTheDocument();
  });

  it("renders Codex fields for codex appId", () => {
    render(<ProviderForm {...defaultProps} appId="codex" />);

    expect(screen.getByTestId("codex-fields")).toBeInTheDocument();
    expect(screen.queryByTestId("claude-fields")).not.toBeInTheDocument();
  });

  it("renders Gemini fields for gemini appId", () => {
    render(<ProviderForm {...defaultProps} appId="gemini" />);

    expect(screen.getByTestId("gemini-fields")).toBeInTheDocument();
    expect(screen.queryByTestId("claude-fields")).not.toBeInTheDocument();
  });

  it("renders config editor based on appId", () => {
    const { rerender } = render(<ProviderForm {...defaultProps} appId="claude" />);
    expect(screen.getByTestId("common-config-editor")).toBeInTheDocument();

    rerender(<ProviderForm {...defaultProps} appId="codex" />);
    expect(screen.getByTestId("codex-config-editor")).toBeInTheDocument();

    rerender(<ProviderForm {...defaultProps} appId="gemini" />);
    expect(screen.getByTestId("gemini-config-editor")).toBeInTheDocument();
  });

  it("shows buttons by default", () => {
    render(<ProviderForm {...defaultProps} />);

    expect(screen.getByRole("button", { name: "common.cancel" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
  });

  it("hides buttons when showButtons is false", () => {
    render(<ProviderForm {...defaultProps} showButtons={false} />);

    expect(screen.queryByRole("button", { name: "common.cancel" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Save" })).not.toBeInTheDocument();
  });

  it("calls onCancel when cancel button clicked", async () => {
    const user = userEvent.setup();
    const onCancel = vi.fn();

    render(<ProviderForm {...defaultProps} onCancel={onCancel} />);

    await user.click(screen.getByRole("button", { name: "common.cancel" }));

    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("changes preset selection", async () => {
    const user = userEvent.setup();

    render(<ProviderForm {...defaultProps} />);

    expect(screen.getByTestId("selected-preset")).toHaveTextContent("custom");

    await user.click(screen.getByText("Select Preset"));

    await waitFor(() => {
      expect(screen.getByTestId("selected-preset")).toHaveTextContent("claude-0");
    });
  });

  it("uses initial data in edit mode", () => {
    render(
      <ProviderForm
        {...defaultProps}
        initialData={{
          name: "My Provider",
          websiteUrl: "https://example.com",
          notes: "Some notes",
          settingsConfig: { env: { API_KEY: "test" } },
        }}
      />
    );

    expect(screen.getByTestId("name-input")).toHaveValue("My Provider");
    expect(screen.getByTestId("website-input")).toHaveValue("https://example.com");
  });

  it("submits form with correct values", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();

    render(<ProviderForm {...defaultProps} onSubmit={onSubmit} />);

    const nameInput = screen.getByTestId("name-input");
    await user.clear(nameInput);
    await user.type(nameInput, "New Provider");

    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalled();
    });

    const submittedData = onSubmit.mock.calls[0][0];
    expect(submittedData.name).toBe("New Provider");
  });

  it("has form id for external submission", () => {
    render(<ProviderForm {...defaultProps} />);

    const form = document.getElementById("provider-form");
    expect(form).toBeInTheDocument();
    expect(form?.tagName).toBe("FORM");
  });
});
