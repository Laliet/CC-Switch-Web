import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { ThemeSettings } from "@/components/settings/ThemeSettings";

const tMock = vi.fn((key: string) => key);

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: tMock }),
}));

type Theme = "light" | "dark" | "system";

let themeState: Theme = "light";
const setThemeMock = vi.fn((nextTheme: Theme) => {
  themeState = nextTheme;
});
const useThemeMock = vi.fn(() => ({
  theme: themeState,
  setTheme: setThemeMock,
}));

vi.mock("@/components/theme-provider", () => ({
  useTheme: () => useThemeMock(),
}));

const getButtons = () => ({
  light: screen.getByRole("button", { name: "settings.themeLight" }),
  dark: screen.getByRole("button", { name: "settings.themeDark" }),
  system: screen.getByRole("button", { name: "settings.themeSystem" }),
});

beforeEach(() => {
  themeState = "light";
  setThemeMock.mockClear();
  useThemeMock.mockClear();
  tMock.mockClear();
});

describe("ThemeSettings", () => {
  it("renders three theme buttons", () => {
    render(<ThemeSettings />);

    const buttons = getButtons();
    expect(screen.getAllByRole("button")).toHaveLength(3);
    expect(buttons.light).toBeInTheDocument();
    expect(buttons.dark).toBeInTheDocument();
    expect(buttons.system).toBeInTheDocument();
  });

  it("shows active state on the current theme button", () => {
    themeState = "dark";
    render(<ThemeSettings />);

    const buttons = getButtons();
    expect(buttons.dark).toHaveClass("shadow-sm");
    expect(buttons.light).not.toHaveClass("shadow-sm");
    expect(buttons.system).not.toHaveClass("shadow-sm");
  });

  it("calls setTheme when clicking theme buttons", () => {
    render(<ThemeSettings />);

    const buttons = getButtons();
    fireEvent.click(buttons.dark);
    fireEvent.click(buttons.system);
    fireEvent.click(buttons.light);

    expect(setThemeMock).toHaveBeenCalledTimes(3);
    expect(setThemeMock).toHaveBeenNthCalledWith(1, "dark");
    expect(setThemeMock).toHaveBeenNthCalledWith(2, "system");
    expect(setThemeMock).toHaveBeenNthCalledWith(3, "light");
  });

  it("switches active state across all themes", () => {
    const { rerender } = render(<ThemeSettings />);

    let buttons = getButtons();
    expect(buttons.light).toHaveClass("shadow-sm");
    expect(buttons.dark).not.toHaveClass("shadow-sm");
    expect(buttons.system).not.toHaveClass("shadow-sm");

    fireEvent.click(buttons.dark);
    rerender(<ThemeSettings />);

    buttons = getButtons();
    expect(buttons.dark).toHaveClass("shadow-sm");
    expect(buttons.light).not.toHaveClass("shadow-sm");
    expect(buttons.system).not.toHaveClass("shadow-sm");

    fireEvent.click(buttons.system);
    rerender(<ThemeSettings />);

    buttons = getButtons();
    expect(buttons.system).toHaveClass("shadow-sm");
    expect(buttons.light).not.toHaveClass("shadow-sm");
    expect(buttons.dark).not.toHaveClass("shadow-sm");

    fireEvent.click(buttons.light);
    rerender(<ThemeSettings />);

    buttons = getButtons();
    expect(buttons.light).toHaveClass("shadow-sm");
    expect(buttons.dark).not.toHaveClass("shadow-sm");
    expect(buttons.system).not.toHaveClass("shadow-sm");
  });
});
