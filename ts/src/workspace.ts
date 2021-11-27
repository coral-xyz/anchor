import camelCase from "camelcase";
import * as toml from "toml";
import { PublicKey } from "@solana/web3.js";
import { Program } from "./program";
import { Idl } from "./idl";
import Provider from "./provider";
import { isBrowser } from "./utils/common";

let _populatedWorkspace = false;

/**
 * The `workspace` namespace provides a convenience API to automatically
 * search for and deserialize [[Program]] objects defined by compiled IDLs
 * in an Anchor workspace.
 *
 * This API is for Node only.
 */
const workspace = new Proxy({} as any, {
  async get(workspaceCache: { [key: string]: Program }, programName: string, provider: Provider) {
    if (isBrowser) {
      throw new Error("Workspaces aren't available in the browser");
    }

    const fs = await import("fs");
    const process = await import("process");

    if (!_populatedWorkspace) {
      const path = await import("path");

      let projectRoot: string | undefined = process.cwd();
      while (!fs.existsSync(path.join(projectRoot, "Anchor.toml"))) {
        const parentDir = path.dirname(projectRoot);
        if (parentDir === projectRoot) {
          projectRoot = undefined;
        }
        projectRoot = parentDir;
      }

      if (projectRoot === undefined) {
        throw new Error("Could not find workspace root.");
      }

      const idlFolder = `${projectRoot}/target/idl`;
      if (!fs.existsSync(idlFolder)) {
        throw new Error(
          `${idlFolder} doesn't exist. Did you use "anchor build"?`
        );
      }

      const idlMap = new Map<string, Idl>();
      fs.readdirSync(idlFolder).forEach((file) => {
        const filePath = `${idlFolder}/${file}`;
        const idlStr = fs.readFileSync(filePath);
        const idl = JSON.parse(idlStr.toString());
        idlMap.set(idl.name, idl);
        const name = camelCase(idl.name, { pascalCase: true });
        if (idl.metadata && idl.metadata.address) {
          workspaceCache[name] = new Program(
            idl,
            new PublicKey(idl.metadata.address),
            provider
          );
        }
      });

      // Override the workspace programs if the user put them in the config.
      const anchorToml = toml.parse(
        fs.readFileSync(path.join(projectRoot, "Anchor.toml"), "utf-8")
      );
      const clusterId = anchorToml.provider.cluster;
      if (anchorToml.programs && anchorToml.programs[clusterId]) {
        await attachWorkspaceOverride(
          workspaceCache,
          anchorToml.programs[clusterId],
          idlMap,
          provider
        );
      }

      _populatedWorkspace = true;
    }

    return workspaceCache[programName];
  },
});

async function attachWorkspaceOverride(
  workspaceCache: { [key: string]: Program },
  overrideConfig: { [key: string]: string | { address: string; idl?: string } },
  idlMap: Map<string, Idl>,
  provider: Provider,
) {
  const configKeys = Object.keys(overrideConfig);

  for (const programName of configKeys) {
    const wsProgramName = camelCase(programName, { pascalCase: true });
    const entry = overrideConfig[programName];
    const overrideAddress = new PublicKey(
      typeof entry === "string" ? entry : entry.address
    );
    let idl = idlMap.get(programName);
    if (typeof entry !== "string" && entry.idl) {
      const fs = await import('fs');
      idl = JSON.parse(fs.readFileSync(entry.idl, "utf-8").toString());
    }
    if (!idl) {
      throw new Error(`Error loading workspace IDL for ${programName}`);
    }
    workspaceCache[wsProgramName] = new Program(idl, overrideAddress, provider);
  }
}

export default workspace;
