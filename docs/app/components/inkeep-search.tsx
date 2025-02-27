"use client";
import type { SharedProps } from "fumadocs-ui/components/dialog/search";
import * as inkeepEmbed from "@inkeep/uikit-js";
import { useEffect, useRef } from "react";
import { settings } from "../inkeep-settings";

export default function CustomDialog(props: SharedProps) {
  const { open, onOpenChange } = props;
  const CustomModalRef = useRef<any>(null);

  useEffect(() => {
    const loadInkeepEmbed = async () => {
      const inkeep = inkeepEmbed.Inkeep(settings.baseSettings);
      CustomModalRef.current = inkeep.embed({
        componentType: "CustomTrigger",
        colorModeSync: {
          observedElement: document.documentElement,
          isDarkModeCallback: (el: any) => {
            return el.classList.contains("dark");
          },
          colorModeAttribute: "class",
        },
        properties: {
          isOpen: open,
          onClose: () => {
            onOpenChange(false);
          },
          ...settings,
        },
      });
    };

    loadInkeepEmbed();
  }, []);

  useEffect(() => {
    if (CustomModalRef.current) {
      CustomModalRef.current.render({
        isOpen: open,
      });
    }
  }, [open]);

  return null;
}
