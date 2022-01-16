# 安装
## Rust

去 [这里](https://www.rust-lang.org/tools/install) 安装 Rust.

## Solana

去 [这里](https://docs.solana.com/cli/install-solana-cli-tools) 安装 Solana.

## Yarn

去 [这里](https://yarnpkg.com/getting-started/install) 安装 Yarn.

## Anchor

### 通过 pre-build binary 在 x86_64 Linux 安装

Anchor 的二进制文件可以通过 NPM package [`@project-serum/anchor-cli`](https://www.npmjs.com/package/@project-serum/anchor-cli)安装. 目前只支持 `x86_64` Linux, 其他的OS你需要从源代码手动构建。

### 从源代码手动构建（其他操作系统）

目前我们可以通过Cargo来安装CLI命令行。

```
cargo install --git https://github.com/project-serum/anchor --tag v0.20.1 anchor-cli --locked
```

在Linux系统，如果cargo install失败，你需要安装额外的依赖。在Ubuntu,

```
sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev
```

现在，验证命令行CLI已经正确安装。

```
anchor --version
```