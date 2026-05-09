import { useMemo, useState, useEffect } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import { useTranslation } from "react-i18next";
import type {
  DraggableAttributes,
  DraggableSyntheticListeners,
} from "@dnd-kit/core";
import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { cn } from "@/lib/utils";
import { ProviderActions } from "@/components/providers/ProviderActions";
import { ProviderIcon } from "@/components/ProviderIcon";
import UsageFooter from "@/components/UsageFooter";
import SubscriptionQuotaFooter from "@/components/SubscriptionQuotaFooter";
import CopilotQuotaFooter from "@/components/CopilotQuotaFooter";
import CodexOauthQuotaFooter from "@/components/CodexOauthQuotaFooter";
import { PROVIDER_TYPES } from "@/config/constants";
import { isHermesReadOnlyProvider } from "@/config/hermesProviderPresets";
import { ProviderHealthBadge } from "@/components/providers/ProviderHealthBadge";
import { FailoverPriorityBadge } from "@/components/providers/FailoverPriorityBadge";
import { extractCodexBaseUrl } from "@/utils/providerConfigUtils";
import { useProviderHealth } from "@/lib/query/failover";
import { useUsageQuery } from "@/lib/query/queries";

interface DragHandleProps {
  attributes: DraggableAttributes;
  listeners: DraggableSyntheticListeners;
  isDragging: boolean;
}

interface ProviderCardProps {
  provider: Provider;
  isCurrent: boolean;
  appId: AppId;
  isInConfig?: boolean; // OpenCode: 是否已添加到 opencode.json
  isOmo?: boolean;
  isOmoSlim?: boolean;
  onSwitch: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onRemoveFromConfig?: (provider: Provider) => void;
  onDisableOmo?: () => void;
  onDisableOmoSlim?: () => void;
  onConfigureUsage: (provider: Provider) => void;
  onOpenWebsite: (url: string) => void;
  onDuplicate: (provider: Provider) => void;
  onTest?: (provider: Provider) => void;
  onOpenTerminal?: (provider: Provider) => void;
  isTesting?: boolean;
  isProxyRunning: boolean;
  isProxyTakeover?: boolean; // 代理接管模式（Live配置已被接管，切换为热切换）
  dragHandleProps?: DragHandleProps;
  isAutoFailoverEnabled?: boolean; // 是否开启自动故障转移
  failoverPriority?: number; // 故障转移优先级（1 = P1, 2 = P2, ...）
  isInFailoverQueue?: boolean; // 是否在故障转移队列中
  onToggleFailover?: (enabled: boolean) => void; // 切换故障转移队列
  activeProviderId?: string; // 代理当前实际使用的供应商 ID（用于故障转移模式下标注绿色边框）
  // OpenClaw: default model
  isDefaultModel?: boolean;
  onSetAsDefault?: () => void;
}

/** 判断是否为官方供应商（无自定义 base URL / API key，直连官方 API） */
function isOfficialProvider(provider: Provider, appId: AppId): boolean {
  const config = provider.settingsConfig as Record<string, any>;
  if (appId === "claude") {
    const baseUrl = config?.env?.ANTHROPIC_BASE_URL;
    return !baseUrl || (typeof baseUrl === "string" && baseUrl.trim() === "");
  }
  if (appId === "codex") {
    // 无 OPENAI_API_KEY → 使用 Codex CLI 内置 OAuth（官方）
    const apiKey = config?.auth?.OPENAI_API_KEY;
    return !apiKey || (typeof apiKey === "string" && apiKey.trim() === "");
  }
  if (appId === "gemini") {
    // 无 GEMINI_API_KEY 且无 GOOGLE_GEMINI_BASE_URL → Google OAuth 官方模式
    const apiKey = config?.env?.GEMINI_API_KEY;
    const baseUrl = config?.env?.GOOGLE_GEMINI_BASE_URL;
    return (
      (!apiKey || (typeof apiKey === "string" && apiKey.trim() === "")) &&
      (!baseUrl || (typeof baseUrl === "string" && baseUrl.trim() === ""))
    );
  }
  return false;
}

const extractApiUrl = (provider: Provider, fallbackText: string) => {
  if (provider.notes?.trim()) {
    return provider.notes.trim();
  }

  if (provider.websiteUrl) {
    return provider.websiteUrl;
  }

  const config = provider.settingsConfig;

  if (config && typeof config === "object") {
    const envBase =
      (config as Record<string, any>)?.env?.ANTHROPIC_BASE_URL ||
      (config as Record<string, any>)?.env?.GOOGLE_GEMINI_BASE_URL;
    if (typeof envBase === "string" && envBase.trim()) {
      return envBase;
    }

    const baseUrl = (config as Record<string, any>)?.config;

    if (typeof baseUrl === "string" && baseUrl.includes("base_url")) {
      const extractedBaseUrl = extractCodexBaseUrl(baseUrl);
      if (extractedBaseUrl) {
        return extractedBaseUrl;
      }
    }
  }

  return fallbackText;
};

export function ProviderCard({
  provider,
  isCurrent,
  appId,
  isInConfig = true,
  isOmo = false,
  isOmoSlim = false,
  onSwitch,
  onEdit,
  onDelete,
  onRemoveFromConfig,
  onDisableOmo,
  onDisableOmoSlim,
  onConfigureUsage,
  onOpenWebsite,
  onDuplicate,
  onTest,
  onOpenTerminal,
  isTesting,
  isProxyRunning,
  isProxyTakeover = false,
  dragHandleProps,
  isAutoFailoverEnabled = false,
  failoverPriority,
  isInFailoverQueue = false,
  onToggleFailover,
  activeProviderId,
  // OpenClaw: default model
  isDefaultModel,
  onSetAsDefault,
}: ProviderCardProps) {
  const { t } = useTranslation();

  // OMO and OMO Slim share the same card behavior
  const isAnyOmo = isOmo || isOmoSlim;
  const handleDisableAnyOmo = isOmoSlim ? onDisableOmoSlim : onDisableOmo;
  const isAdditiveMode = appId === "opencode" && !isAnyOmo;

  const { data: health } = useProviderHealth(provider.id, appId);

  const fallbackUrlText = t("provider.notConfigured", {
    defaultValue: "未配置接口地址",
  });

  const displayUrl = useMemo(() => {
    return extractApiUrl(provider, fallbackUrlText);
  }, [provider, fallbackUrlText]);

  const isClickableUrl = useMemo(() => {
    if (provider.notes?.trim()) {
      return false;
    }
    if (displayUrl === fallbackUrlText) {
      return false;
    }
    return true;
  }, [provider.notes, displayUrl, fallbackUrlText]);

  const usageEnabled = provider.meta?.usage_script?.enabled ?? false;
  const isOfficial = isOfficialProvider(provider, appId);
  const isOfficialBlockedByProxy =
    isProxyTakeover && (provider.category === "official" || isOfficial);
  const isCopilot =
    provider.meta?.providerType === PROVIDER_TYPES.GITHUB_COPILOT ||
    provider.meta?.usage_script?.templateType === "github_copilot";
  // Hermes v12+ overlay entries live under the `providers:` dict and are
  // read-only here — writes have to go through Hermes Web UI.
  const isHermesReadOnly =
    appId === "hermes" && isHermesReadOnlyProvider(provider.settingsConfig);
  const isCodexOauth =
    provider.meta?.providerType === PROVIDER_TYPES.CODEX_OAUTH;

  // 获取用量数据以判断是否有多套餐
  // 累加模式应用（OpenCode/OpenClaw/Hermes）：使用 isInConfig 代替 isCurrent
  const shouldAutoQuery =
    appId === "opencode" || appId === "openclaw" || appId === "hermes"
      ? isInConfig
      : isCurrent;
  const autoQueryInterval = shouldAutoQuery
    ? provider.meta?.usage_script?.autoQueryInterval || 0
    : 0;

  const { data: usage } = useUsageQuery(provider.id, appId, {
    enabled: usageEnabled,
    autoQueryInterval,
  });

  const isTokenPlan =
    provider.meta?.usage_script?.templateType === "token_plan";
  const hasMultiplePlans =
    usage?.success && usage.data && usage.data.length > 1 && !isTokenPlan;

  const [isExpanded, setIsExpanded] = useState(false);

  useEffect(() => {
    if (hasMultiplePlans) {
      setIsExpanded(true);
    }
  }, [hasMultiplePlans]);

  const handleOpenWebsite = () => {
    if (!isClickableUrl) {
      return;
    }
    onOpenWebsite(displayUrl);
  };

  // 判断是否是"当前使用中"的供应商
  // - OMO/OMO Slim 供应商：使用 isCurrent
  // - OpenClaw：使用默认模型归属的 provider 作为当前项（蓝色边框）
  // - OpenCode（非 OMO）：不存在"当前"概念，返回 false
  // - 故障转移模式：代理实际使用的供应商（activeProviderId）
  // - 普通模式：isCurrent
  const isActiveProvider = isAnyOmo
    ? isCurrent
    : appId === "openclaw"
      ? Boolean(isDefaultModel)
      : appId === "opencode"
        ? false
        : isAutoFailoverEnabled
          ? activeProviderId === provider.id
          : isCurrent;

  const shouldUseGreen = !isAnyOmo && isProxyTakeover && isActiveProvider;
  const hasPersistentConfigHighlight = isAdditiveMode && isInConfig;
  const shouldUseBlue =
    (isAnyOmo && isActiveProvider) ||
    (!isAnyOmo &&
      !isProxyTakeover &&
      (isActiveProvider || hasPersistentConfigHighlight));

  return (
    <div
      className={cn(
        "claude-card group flex items-center gap-4",
        isActiveProvider && "active",
        shouldUseGreen && "border-emerald-500/50",
        shouldUseBlue && "border-primary/50",
        dragHandleProps?.isDragging && "cursor-grabbing",
      )}
    >
      <div className="w-12 h-12 rounded-full bg-muted flex items-center justify-center flex-shrink-0">
        <ProviderIcon
          icon={provider.icon}
          name={provider.name}
          color={provider.iconColor}
          size={24}
        />
      </div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <h3 className="text-[15px] font-medium text-foreground">{provider.name}</h3>

          {isOmo && (
            <span className="inline-flex items-center bg-orange-100 text-orange-700 dark:bg-orange-900/40 dark:text-orange-400 px-2 py-0.5 text-xs font-medium rounded-md">
              OMO
            </span>
          )}

          {isOmoSlim && (
            <span className="inline-flex items-center bg-orange-100 text-orange-700 dark:bg-orange-900/40 dark:text-orange-400 px-2 py-0.5 text-xs font-medium rounded-md">
              Slim
            </span>
          )}

          {isProxyRunning && isInFailoverQueue && health && (
            <ProviderHealthBadge
              consecutiveFailures={health.consecutive_failures}
            />
          )}

          {isAutoFailoverEnabled &&
            isInFailoverQueue &&
            failoverPriority && (
              <FailoverPriorityBadge priority={failoverPriority} />
            )}

          {provider.category === "third_party" &&
            provider.meta?.isPartner && (
              <span
                className="text-yellow-500 font-bold"
                title={t("provider.officialPartner", {
                  defaultValue: "官方合作伙伴",
                })}
              >
                ⭐
              </span>
            )}

          {isHermesReadOnly && (
            <span
              className="inline-flex items-center bg-muted text-muted-foreground px-2 py-0.5 text-xs font-medium rounded"
              title={t("provider.managedByHermesHint", {
                defaultValue: "由 Hermes 管理，请在 Hermes Web UI 中编辑",
              })}
            >
              {t("provider.managedByHermes", {
                defaultValue: "Hermes Managed",
              })}
            </span>
          )}
        </div>

        {displayUrl && (
          <button
            type="button"
            onClick={handleOpenWebsite}
            className={cn(
              "inline-flex items-center text-[13px] max-w-[280px] font-mono mt-1 text-muted-foreground",
              isClickableUrl
                ? "text-primary/70 hover:text-primary hover:underline cursor-pointer transition-colors"
                : "cursor-default",
            )}
            title={displayUrl}
            disabled={!isClickableUrl}
          >
            <span className="truncate">{displayUrl}</span>
          </button>
        )}
      </div>

      <div className="flex items-center ml-auto min-w-0 gap-3">
        <div className="ml-auto">
          <div className="flex items-center gap-2">
            {isCopilot ? (
              <CopilotQuotaFooter
                meta={provider.meta}
                inline={true}
                isCurrent={isCurrent}
              />
            ) : isCodexOauth ? (
              <CodexOauthQuotaFooter
                meta={provider.meta}
                inline={true}
                isCurrent={isCurrent}
              />
            ) : isOfficial ? (
              <SubscriptionQuotaFooter
                appId={appId}
                inline={true}
                isCurrent={isCurrent}
              />
            ) : hasMultiplePlans ? (
              <div className="flex items-center gap-2 text-xs font-medium">
                <span className="font-medium">
                  {t("usage.multiplePlans", {
                    count: usage?.data?.length || 0,
                    defaultValue: `${usage?.data?.length || 0} 个套餐`,
                  })}
                </span>
              </div>
            ) : (
              <UsageFooter
                provider={provider}
                providerId={provider.id}
                appId={appId}
                usageEnabled={usageEnabled}
                isCurrent={isCurrent}
                isInConfig={isInConfig}
                inline={true}
              />
            )}
            {hasMultiplePlans && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setIsExpanded(!isExpanded);
                }}
                className="p-1 rounded-md hover:bg-muted transition-colors text-muted-foreground flex-shrink-0"
                title={
                  isExpanded
                    ? t("usage.collapse", { defaultValue: "收起" })
                    : t("usage.expand", { defaultValue: "展开" })
                }
              >
                {isExpanded ? (
                  <ChevronUp size={14} />
                ) : (
                  <ChevronDown size={14} />
                )}
              </button>
            )}
          </div>
        </div>

        <div className="flex items-center gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 group-hover:pointer-events-auto group-focus-within:pointer-events-auto transition-all duration-150">
          <ProviderActions
            appId={appId}
            isCurrent={isCurrent}
            isInConfig={isInConfig}
            isTesting={isTesting}
            isProxyTakeover={isProxyTakeover}
            isOfficialBlockedByProxy={isOfficialBlockedByProxy}
            isReadOnly={isHermesReadOnly}
            isOmo={isAnyOmo}
            onSwitch={() => onSwitch(provider)}
            onEdit={() => onEdit(provider)}
            onDuplicate={() => onDuplicate(provider)}
            onTest={
              onTest && !isOfficial && !isCopilot && !isCodexOauth
                ? () => onTest(provider)
                : undefined
            }
            onConfigureUsage={
              isOfficial || isCopilot || isCodexOauth
                ? undefined
                : () => onConfigureUsage(provider)
            }
            onDelete={() => onDelete(provider)}
            onRemoveFromConfig={
              onRemoveFromConfig
                ? () => onRemoveFromConfig(provider)
                : undefined
            }
            onDisableOmo={handleDisableAnyOmo}
            onOpenTerminal={
              onOpenTerminal ? () => onOpenTerminal(provider) : undefined
            }
            isAutoFailoverEnabled={isAutoFailoverEnabled}
            isInFailoverQueue={isInFailoverQueue}
            onToggleFailover={onToggleFailover}
            isDefaultModel={isDefaultModel}
            onSetAsDefault={onSetAsDefault}
          />
        </div>
      </div>

      {isExpanded && hasMultiplePlans && (
        <div className="mt-4 pt-4 border-t">
          <UsageFooter
            provider={provider}
            providerId={provider.id}
            appId={appId}
            usageEnabled={usageEnabled}
            isCurrent={isCurrent}
            isInConfig={isInConfig}
            inline={false}
          />
        </div>
      )}
    </div>
  );
}
