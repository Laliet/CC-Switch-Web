import React from "react";
import { RefreshCw, AlertCircle, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";
import { type AppId } from "@/lib/api";
import { useUsageQuery } from "@/lib/query/queries";
import { UsageData, Provider } from "@/types";

interface UsageFooterProps {
  provider: Provider;
  providerId: string;
  appId: AppId;
  usageEnabled: boolean; // 是否启用了用量查询
  isCurrent: boolean; // 是否为当前激活的供应商
  backupProviderId?: string | null; // 备用供应商 ID，用于自动故障切换
  onAutoFailover?: (targetId: string) => void; // 自动故障切换回调
  inline?: boolean; // 是否内联显示（在按钮左侧）
  autoTokenApplied?: boolean; // 是否已自动应用前端注入的 API/CSRF Token
}

const UsageFooter: React.FC<UsageFooterProps> = ({
  provider,
  providerId,
  appId,
  usageEnabled,
  isCurrent,
  backupProviderId = null,
  onAutoFailover,
  inline = false,
  autoTokenApplied = false,
}) => {
  const { t } = useTranslation();

  // 统一的用量查询（自动查询仅对当前激活的供应商启用）
  const autoQueryInterval = isCurrent
    ? provider.meta?.usage_script?.autoQueryInterval || 0
    : 0;

  const {
    data: usage,
    isFetching: loading,
    lastQueriedAt,
    refetch,
  } = useUsageQuery(providerId, appId, {
    enabled: usageEnabled,
    autoQueryInterval,
  });

  // 自动故障切换：当前供应商查询失败时切到备用
  const failoverAttemptRef = React.useRef<string | null>(null);
  React.useEffect(() => {
    if (
      !isCurrent ||
      !usageEnabled ||
      !backupProviderId ||
      providerId === backupProviderId ||
      !usage
    ) {
      failoverAttemptRef.current = null;
      return;
    }

    if (usage.success) {
      failoverAttemptRef.current = null;
      return;
    }

    if (failoverAttemptRef.current === providerId) return;
    failoverAttemptRef.current = providerId;
    onAutoFailover?.(backupProviderId);
  }, [
    backupProviderId,
    isCurrent,
    onAutoFailover,
    providerId,
    usage,
    usageEnabled,
  ]);

  // 🆕 定期更新当前时间，用于刷新相对时间显示
  const [now, setNow] = React.useState(Date.now());

  React.useEffect(() => {
    if (!lastQueriedAt) return;

    // 每30秒更新一次当前时间，触发相对时间显示的刷新
    const interval = setInterval(() => {
      setNow(Date.now());
    }, 30000); // 30秒

    return () => clearInterval(interval);
  }, [lastQueriedAt]);

  // 只在启用用量查询且有数据时显示
  if (!usageEnabled || !usage) return null;

  // 错误状态
  if (!usage.success) {
    if (inline) {
      return (
        <div className="flex items-center gap-2 text-xs">
          <div className="flex items-center gap-1.5 text-red-500 dark:text-red-400">
            <AlertCircle size={12} />
            <span>{t("usage.queryFailed")}</span>
          </div>
          <button
            onClick={() => refetch()}
            disabled={loading}
            className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors disabled:opacity-50 flex-shrink-0"
            title={t("usage.refreshUsage")}
          >
            <RefreshCw size={12} className={loading ? "animate-spin" : ""} />
          </button>
        </div>
      );
    }

    return (
      <div className="mt-3 pt-3 border-t border-border-default ">
        <div className="flex items-center justify-between gap-2 text-xs">
          <div className="flex items-center gap-2 text-red-500 dark:text-red-400">
            <AlertCircle size={14} />
            <span>{usage.error || t("usage.queryFailed")}</span>
          </div>

          {/* 刷新按钮 */}
          <button
            onClick={() => refetch()}
            disabled={loading}
            className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors disabled:opacity-50 flex-shrink-0"
            title={t("usage.refreshUsage")}
          >
            <RefreshCw size={12} className={loading ? "animate-spin" : ""} />
          </button>
        </div>
      </div>
    );
  }

  const usageDataList = usage.data || [];

  // 无数据时不显示
  if (usageDataList.length === 0) return null;

  const autoTokenApplied =
    typeof window !== "undefined" &&
    Boolean((window as any).__CC_SWITCH_TOKENS__?.apiToken);

  // 内联模式：仅显示第一个套餐的核心数据（分上下两行）
  if (inline) {
    const firstUsage = usageDataList[0];
    const remainingValue =
      firstUsage.remaining !== undefined
        ? firstUsage.remaining
        : firstUsage.total !== undefined && firstUsage.used !== undefined
          ? firstUsage.total - firstUsage.used
          : undefined;
    const usedValue =
      firstUsage.used !== undefined
        ? firstUsage.used
        : firstUsage.total !== undefined && remainingValue !== undefined
          ? firstUsage.total - remainingValue
          : undefined;
    const isExpired = firstUsage.isValid === false;

    return (
      <div className="flex flex-col gap-1 text-xs flex-shrink-0">
        {/* 第一行：刷新时间 + 刷新按钮 */}
        <div className="flex items-center gap-2 justify-end">
          {autoTokenApplied && (
            <span className="rounded-full bg-emerald-50 px-1.5 py-0.5 text-[10px] font-medium text-emerald-700 ring-1 ring-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-200 dark:ring-emerald-800">
              {t("usage.autoToken")}
            </span>
          )}
          {/* 上次查询时间 */}
          {lastQueriedAt && (
            <span className="text-[10px] text-gray-400 dark:text-gray-500 flex items-center gap-1">
              <Clock size={10} />
              {formatRelativeTime(lastQueriedAt, now, t)}
            </span>
          )}

          {/* 刷新按钮 */}
          <button
            onClick={() => refetch()}
            disabled={loading}
            className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors disabled:opacity-50 flex-shrink-0"
            title={t("usage.refreshUsage")}
          >
            <RefreshCw size={12} className={loading ? "animate-spin" : ""} />
          </button>
        </div>

        {/* 第二行：已用 + 剩余 + 单位 */}
        <div className="flex items-center gap-2">
          {/* 已用 */}
          {usedValue !== undefined && (
            <div className="flex items-center gap-0.5">
              <span className="text-gray-500 dark:text-gray-400">
                {t("usage.used")}
              </span>
              <span className="tabular-nums text-gray-600 dark:text-gray-400 font-medium">
                {usedValue.toFixed(2)}
              </span>
            </div>
          )}

          {/* 剩余 */}
          {remainingValue !== undefined && (
            <div className="flex items-center gap-0.5">
              <span className="text-gray-500 dark:text-gray-400">
                {t("usage.remaining")}
              </span>
              <span
                className={`font-semibold tabular-nums ${
                  isExpired
                    ? "text-red-500 dark:text-red-400"
                    : remainingValue <
                        (firstUsage.total || remainingValue) * 0.1
                      ? "text-orange-500 dark:text-orange-400"
                      : "text-green-600 dark:text-green-400"
                }`}
              >
                {remainingValue.toFixed(2)}
              </span>
            </div>
          )}

          {/* 单位 */}
          {firstUsage.unit && (
            <span className="text-gray-500 dark:text-gray-400">
              {firstUsage.unit}
            </span>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="mt-3 pt-3 border-t border-border-default ">
      {/* 标题行：包含刷新按钮和自动查询时间 */}
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs text-gray-500 dark:text-gray-400 font-medium">
          {t("usage.planUsage")}
        </span>
        <div className="flex items-center gap-2">
          {autoTokenApplied && (
            <span className="rounded-full bg-emerald-50 px-2 py-0.5 text-[11px] font-medium text-emerald-700 ring-1 ring-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-200 dark:ring-emerald-800">
              {t("usage.autoToken")}
            </span>
          )}
          {/* 自动查询时间提示 */}
          {lastQueriedAt && (
            <span className="text-[10px] text-gray-400 dark:text-gray-500 flex items-center gap-1">
              <Clock size={10} />
              {formatRelativeTime(lastQueriedAt, now, t)}
            </span>
          )}
          <button
            onClick={() => refetch()}
            disabled={loading}
            className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors disabled:opacity-50"
            title={t("usage.refreshUsage")}
          >
            <RefreshCw size={12} className={loading ? "animate-spin" : ""} />
          </button>
        </div>
      </div>

      {/* 套餐列表 */}
      <div className="flex flex-col gap-3">
        {usageDataList.map((usageData, index) => (
          <UsagePlanItem key={index} data={usageData} />
        ))}
      </div>
    </div>
  );
};

// 单个套餐数据展示组件
const UsagePlanItem: React.FC<{ data: UsageData }> = ({ data }) => {
  const { t } = useTranslation();
  const {
    planName,
    extra,
    isValid,
    invalidMessage,
    total,
    used,
    remaining,
    unit,
  } = data;

  const remainingValue =
    remaining !== undefined
      ? remaining
      : total !== undefined && used !== undefined
        ? total - used
        : undefined;
  const usedValue =
    used !== undefined
      ? used
      : total !== undefined && remainingValue !== undefined
        ? total - remainingValue
        : undefined;

  // 判断套餐是否失效（isValid 为 false 或未定义时视为有效）
  const isExpired = isValid === false;

  return (
    <div className="flex items-center gap-3">
      {/* 标题部分：25% */}
      <div
        className="text-xs text-gray-500 dark:text-gray-400 min-w-0"
        style={{ width: "25%" }}
      >
        {planName ? (
          <span
            className={`font-medium truncate block ${isExpired ? "text-red-500 dark:text-red-400" : ""}`}
            title={planName}
          >
            💰 {planName}
          </span>
        ) : (
          <span className="opacity-50">—</span>
        )}
      </div>

      {/* 扩展字段：30% */}
      <div
        className="text-xs text-gray-500 dark:text-gray-400 min-w-0 flex items-center gap-2"
        style={{ width: "30%" }}
      >
        {extra && (
          <span
            className={`truncate ${isExpired ? "text-red-500 dark:text-red-400" : ""}`}
            title={extra}
          >
            {extra}
          </span>
        )}
        {isExpired && (
          <span className="text-red-500 dark:text-red-400 font-medium text-[10px] px-1.5 py-0.5 bg-red-50 dark:bg-red-900/20 rounded flex-shrink-0">
            {invalidMessage || t("usage.invalid")}
          </span>
        )}
      </div>

      {/* 用量信息：45% */}
      <div
        className="flex items-center justify-end gap-2 text-xs flex-shrink-0"
        style={{ width: "45%" }}
      >
        {/* 总额度 */}
        {total !== undefined && (
          <>
            <span className="text-gray-500 dark:text-gray-400">
              {t("usage.total")}
            </span>
            <span className="tabular-nums text-gray-600 dark:text-gray-400">
              {total === -1 ? "∞" : total.toFixed(2)}
            </span>
            <span className="text-gray-400 dark:text-gray-600">|</span>
          </>
        )}

        {/* 已用额度 */}
        {usedValue !== undefined && (
          <>
            <span className="text-gray-500 dark:text-gray-400">
              {t("usage.used")}
            </span>
            <span className="tabular-nums text-gray-600 dark:text-gray-400">
              {usedValue.toFixed(2)}
            </span>
            <span className="text-gray-400 dark:text-gray-600">|</span>
          </>
        )}

        {/* 剩余额度 - 突出显示 */}
        {remainingValue !== undefined && (
          <>
            <span className="text-gray-500 dark:text-gray-400">
              {t("usage.remaining")}
            </span>
            <span
              className={`font-semibold tabular-nums ${
                isExpired
                  ? "text-red-500 dark:text-red-400"
                  : remainingValue < (total || remainingValue) * 0.1
                    ? "text-orange-500 dark:text-orange-400"
                    : "text-green-600 dark:text-green-400"
              }`}
            >
              {remainingValue.toFixed(2)}
            </span>
          </>
        )}

        {unit && (
          <span className="text-gray-500 dark:text-gray-400">{unit}</span>
        )}
      </div>
    </div>
  );
};

// 格式化相对时间
function formatRelativeTime(
  timestamp: number,
  now: number,
  t: (key: string, options?: { count?: number }) => string,
): string {
  const diff = Math.floor((now - timestamp) / 1000); // 秒

  if (diff < 60) {
    return t("usage.justNow");
  } else if (diff < 3600) {
    const minutes = Math.floor(diff / 60);
    return t("usage.minutesAgo", { count: minutes });
  } else if (diff < 86400) {
    const hours = Math.floor(diff / 3600);
    return t("usage.hoursAgo", { count: hours });
  } else {
    const days = Math.floor(diff / 86400);
    return t("usage.daysAgo", { count: days });
  }
}

export default UsageFooter;
