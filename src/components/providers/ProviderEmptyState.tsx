import { Download, Users } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import type { AppId } from "@/lib/api/types";
import { isClaudeFamilyApp } from "@/lib/appFamilies";

interface ProviderEmptyStateProps {
  appId: AppId;
  onCreate?: () => void;
  onImport?: () => void;
}

export function ProviderEmptyState({
  appId,
  onCreate,
  onImport,
}: ProviderEmptyStateProps) {
  const { t } = useTranslation();
  const showSnippetHint =
    isClaudeFamilyApp(appId) || appId === "codex" || appId === "gemini";

  return (
    <div className="claude-card flex flex-col items-center justify-center border-dashed p-10 text-center">
      <div className="w-14 h-14 rounded-full bg-muted flex items-center justify-center mb-4">
        <Users className="h-6 w-6 text-muted-foreground" />
      </div>
      <h3 className="text-[15px] font-medium">{t("provider.noProviders")}</h3>
      <p className="mt-2 max-w-lg text-[13px] text-muted-foreground">
        {t("provider.noProvidersDescription")}
      </p>
      {showSnippetHint && (
        <p className="mt-1 max-w-lg text-[13px] text-muted-foreground">
          {t("provider.noProvidersDescriptionSnippet")}
        </p>
      )}
      <div className="mt-6 flex flex-col gap-2 w-full max-w-xs">
        {onImport && (
          <Button variant="secondary" onClick={onImport} className="w-full text-[14px]">
            <Download className="mr-2 h-4 w-4" />
            {t("provider.importCurrent")}
          </Button>
        )}
        {onCreate && (
          <Button onClick={onCreate} className="w-full text-[14px]">
            <Users className="mr-2 h-4 w-4" />
            {t("provider.addProvider")}
          </Button>
        )}
      </div>
    </div>
  );
}
