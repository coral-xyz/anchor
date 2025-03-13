"use client";
import { RootProvider } from "fumadocs-ui/provider";
import dynamic from "next/dynamic";
import type { ReactNode } from "react";

const SearchDialog = dynamic(
  () => import("@/app/components/inkeep/inkeep-search"),
); // lazy load

export function Provider({ children }: { children: ReactNode }) {
  return (
    <RootProvider
      search={{
        SearchDialog,
      }}
    >
      {children}
    </RootProvider>
  );
}
