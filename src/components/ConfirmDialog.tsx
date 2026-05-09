import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AlertTriangle, Info } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: "destructive" | "info";
  zIndex?: "base" | "nested" | "alert" | "top";
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmText,
  cancelText,
  variant = "destructive",
  zIndex = "alert",
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const { t } = useTranslation();

  const IconComponent = variant === "info" ? Info : AlertTriangle;
  const iconClass =
    variant === "info" ? "h-5 w-5 text-[#FF5A36]" : "h-5 w-5 text-red-500";

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          onCancel();
        }
      }}
    >
      <DialogContent className={cn("max-w-sm", zIndex === "alert" ? "z-[100]" : "")} zIndex={zIndex}>
        <DialogHeader className="space-y-3">
          <DialogTitle className="flex items-center gap-2 text-lg font-semibold">
            <IconComponent className={iconClass} />
            {title}
          </DialogTitle>
          <DialogDescription className="whitespace-pre-line text-sm leading-relaxed text-muted-foreground">
            {message}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="flex gap-2 sm:justify-end">
          <Button variant="outline" onClick={onCancel}>
            {cancelText || t("common.cancel")}
          </Button>
          <Button
            variant={variant === "info" ? "default" : "destructive"}
            onClick={onConfirm}
          >
            {confirmText || t("common.confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
