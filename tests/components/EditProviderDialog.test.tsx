import { render, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { EditProviderDialog } from "@/components/providers/EditProviderDialog";

const tMock = vi.fn((key: string) => key);

const providersApiMock = vi.hoisted(() => ({
  getCurrent: vi.fn(),
}));

const vscodeApiMock = vi.hoisted(() => ({
  getLiveProviderSettings: vi.fn(),
}));

const providerFormMock = vi.hoisted(() => vi.fn());

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: tMock }),
}));

vi.mock("@/lib/api", () => ({
  providersApi: providersApiMock,
  vscodeApi: vscodeApiMock,
}));

vi.mock("@/components/providers/forms/ProviderForm", () => ({
  ProviderForm: (props: any) => {
    providerFormMock(props);
    return <div data-testid="provider-form" />;
  },
}));

vi.mock("@/components/ui/dialog", () => ({
  Dialog: ({ open, children }: any) =>
    open ? <div data-testid="dialog-root">{children}</div> : null,
  DialogContent: ({ children }: any) => <div>{children}</div>,
  DialogHeader: ({ children }: any) => <div>{children}</div>,
  DialogFooter: ({ children }: any) => <div>{children}</div>,
  DialogTitle: ({ children }: any) => <h2>{children}</h2>,
  DialogDescription: ({ children }: any) => <p>{children}</p>,
}));

vi.mock("@/components/ui/button", () => ({
  Button: ({ children, ...rest }: any) => <button {...rest}>{children}</button>,
}));

describe("EditProviderDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    tMock.mockImplementation((key: string) => key);
  });

  it("uses provider snapshot for opencode when live provider fragment is missing", async () => {
    providersApiMock.getCurrent.mockResolvedValue("open-current");
    vscodeApiMock.getLiveProviderSettings.mockResolvedValue({
      $schema: "https://opencode.ai/config.json",
      provider: {
        other: {
          options: {
            baseURL: "https://other.example.com/v1",
            apiKey: "other-key",
          },
        },
      },
    });

    render(
      <EditProviderDialog
        open
        appId="opencode"
        provider={{
          id: "open-current",
          name: "Open Current",
          settingsConfig: {
            options: {
              baseURL: "https://saved.example.com/v1",
              apiKey: "saved-key",
            },
          },
        }}
        onOpenChange={vi.fn()}
        onSubmit={vi.fn()}
      />,
    );

    await waitFor(() => {
      const lastProps = providerFormMock.mock.calls.at(-1)?.[0];
      expect(lastProps?.initialData?.settingsConfig).toEqual({
        options: {
          baseURL: "https://saved.example.com/v1",
          apiKey: "saved-key",
        },
      });
    });
  });

  it("uses live opencode provider fragment when available", async () => {
    providersApiMock.getCurrent.mockResolvedValue("open-current");
    vscodeApiMock.getLiveProviderSettings.mockResolvedValue({
      $schema: "https://opencode.ai/config.json",
      provider: {
        "open-current": {
          options: {
            baseURL: "https://live.example.com/v1",
            apiKey: "live-key",
          },
        },
      },
    });

    render(
      <EditProviderDialog
        open
        appId="opencode"
        provider={{
          id: "open-current",
          name: "Open Current",
          settingsConfig: {
            options: {
              baseURL: "https://saved.example.com/v1",
              apiKey: "saved-key",
            },
          },
        }}
        onOpenChange={vi.fn()}
        onSubmit={vi.fn()}
      />,
    );

    await waitFor(() => {
      const lastProps = providerFormMock.mock.calls.at(-1)?.[0];
      expect(lastProps?.initialData?.settingsConfig).toEqual({
        options: {
          baseURL: "https://live.example.com/v1",
          apiKey: "live-key",
        },
      });
    });
  });
});
