import { useCallback, useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { setWebCredentials } from "@/lib/api/adapter";

function base64EncodeUtf8(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return window.btoa(binary);
}

export interface WebLoginDialogProps {
  open: boolean;
  onLoginSuccess: () => void;
}

export function WebLoginDialog({ open, onLoginSuccess }: WebLoginDialogProps) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!open) return;
    setPassword("");
    setError(null);
    setIsSubmitting(false);
  }, [open]);

  const handleLogin = useCallback(async () => {
    if (isSubmitting) return;

    const trimmed = password.trim();
    if (!trimmed) {
      setError("请输入密码");
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      const encoded = base64EncodeUtf8(`admin:${trimmed}`);
      const response = await fetch("/api/settings", {
        method: "GET",
        credentials: "include",
        headers: {
          Accept: "application/json",
          Authorization: `Basic ${encoded}`,
        },
      });

      if (response.ok) {
        setWebCredentials(trimmed);
        try {
          const tokenResponse = await fetch("/api/system/csrf-token", {
            method: "GET",
            credentials: "include",
            headers: {
              Accept: "application/json",
              Authorization: `Basic ${encoded}`,
            },
          });
          if (tokenResponse.ok) {
            const data = (await tokenResponse.json()) as {
              csrfToken?: string | null;
            };
            if (data?.csrfToken) {
              window.sessionStorage?.setItem(
                "cc-switch-csrf-token",
                data.csrfToken,
              );
            } else {
              window.sessionStorage?.removeItem("cc-switch-csrf-token");
            }
          }
        } catch {
          // ignore
        }
        onLoginSuccess();
        return;
      }

      if (response.status === 401) {
        setError("密码错误");
        return;
      }

      const detail = (await response.text())?.trim();
      setError(detail || `登录失败（${response.status}）`);
    } catch (e) {
      setError((e as Error)?.message || "网络错误");
    } finally {
      setIsSubmitting(false);
    }
  }, [isSubmitting, onLoginSuccess, password]);

  return (
    <Dialog open={open} onOpenChange={() => {}}>
      <DialogContent className="max-w-sm">
        <DialogHeader className="space-y-2">
          <DialogTitle>登录</DialogTitle>
          <DialogDescription>
            用户名固定为 <span className="font-mono">admin</span>
          </DialogDescription>
        </DialogHeader>

        <form
          className="space-y-4 px-6 py-5"
          onSubmit={(e) => {
            e.preventDefault();
            void handleLogin();
          }}
        >
          <div className="space-y-2">
            <Label htmlFor="cc-switch-web-password">密码</Label>
            <Input
              id="cc-switch-web-password"
              type="password"
              autoComplete="current-password"
              autoFocus
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              disabled={isSubmitting}
            />
          </div>

          {error ? (
            <div className="text-sm text-destructive">{error}</div>
          ) : null}

          <DialogFooter className="px-0 py-0 border-0 bg-transparent">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "验证中..." : "登录"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

export default WebLoginDialog;
