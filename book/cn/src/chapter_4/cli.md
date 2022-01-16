# 命令行CLI
我们可以通过命令行CLI来构建和管理Anchor的工作环境.
查看详细的命令和对应的选项列表, 我们可以通过`anchor -h`来查看子命令.

```
anchor-cli

USAGE:
    anchor <SUBCOMMAND>

FLAGS:
    -h, --help       打印帮助信息
    -V, --version    打印版本信息

SUBCOMMANDS:
    build      构建工作环境
    cluster    Cluster集群命令
    deploy     部署工作环境中的每个程序
    expand     扩展工作环境中一个程序的宏（macro）
    help       打印现在这个消息或者对应子命令的帮助信息
    idl        与接口定义交互的命令
    init       初始化一个工作环境
    migrate    运行部署迁移脚本
    new        创建一个新的程序
    test       在localnetwork中运行集成测试
    upgrade    更新单个程序.配置的钱包必须由升级权限（upgrade authority）
    verify     验证链上的字节码和本地的编译结果一致. 需要在一个程序的子文件夹中运行这个命令, 例如, 在包含程序的
               Cargo.toml的文件夹中。
```


## Build构建

```
anchor build
```

在工作环境中构建目标为Solana的BPF运行环境的程序，并且输出IDLs到`target/idl`文件夹.

```
anchor build --verifiable
```

在docker镜像中构建来保证输出的二进制的确定性（假定使用了一个Cargo.lock文件）.这个命令必须在工作环境中一个单独的crate子文件夹中运行. 例如, `programs/<my-program>/`.

## Clust集群

### Clust list集群列表

```
anchor cluster list
```

列出集群端口:

```
集群端口:

* Mainnet - https://solana-api.projectserum.com
* Mainnet - https://api.mainnet-beta.solana.com
* Devnet  - https://api.devnet.solana.com
* Testnet - https://api.testnet.solana.com
```

## 部署Deploy

```
anchor deploy
```

把工作环境中的所有程序部署到配置的网络。

::: 小贴士
这与`solana program deploy`命令不同, 因为每次运行都会生成 *新的* program address.
:::

## 扩展Expand

```
anchor expand
```

如果在一个程序（program）的文件夹中运行, 那么扩展该程序的宏.

如过在工作环境workspace中，但是在程序文件夹外, 那么扩展整个工作环境的宏.

如果带着`--program-name`选项运行, 那么只扩展所选定的程序.

## 接口定义命令Idl

`idl` 子命令用来和接口定义文件interface definition files交互.
推荐的范式是把IDL存在链上, 一个固定的地址, as a function of nothing but the the program's ID. This
allows us to generate clients for a program using nothing but the program ID.

### Idl初始化

```
anchor idl init -f <target/idl/program.json> <program-id>
```

创建一个idl账号, 把给出的 `<target/idl/program.json>`文件写入一个program owned account. 常规情况下, account 的大小是IDL account的两倍, 这个是给未来IDL升级流出空间.

### Idl获取

```
anchor idl fetch -o <out-file.json> <program-id>
```

从当前配置的网络读取一个IDL。 比如, 确保你的
`Anchor.toml`指向`mainnet`集群后运行

```
anchor idl fetch GrAkKfEpTKQuVHG2Y97Y2FF4i7y7Q5AHLK94JBy7Y5yv
```

### Idl权限

```
anchor idl authority <program-id>
```

输出IDL账号的当前authority.也就是当前有权限更新IDL的钱包。

### 删除Idl的authority权限

```
anchor idl erase-authority -p <program-id>
```

删掉IDL 账号的authority权限来禁止更新. 配置的钱包wallet必须是当前的authority.

### 合约接口Idl升级

```
anchor idl upgrade <program-id> -f <target/idl/program.json>
```

升级IDL链上文件到新版的`target/idl/program.json`idl.
配置的钱包wallet必须现在的authority权限所有者.

```
anchor idl set-authority -n <new-authority> -p <program-id>
```

在IDL的账号设置一个新的authority权限. `new-authority`和`program-id`
都必须是base 58编码.

## 项目初始化Init

```
anchor init
```

初始化一个有下面文件结构的项目工作环境。

* `Anchor.toml`: Anchor配置文件.
* `Cargo.toml`: Rust工作环境配置文件.
* `package.json`: JavaScript依赖文件.
* `programs/`: Solana程序crates的文件夹.
* `app/`: 前端代码的文件夹.
* `tests/`: JavaScript集成测试的文件夹.
* `migrations/deploy.js`: 部署脚本.

## 迁移Migrate

```
anchor migrate
```

运行 `migrations/deploy.js`中的部署脚本, 注入由`Anchor.toml`中配置好的provider. 例如,

```javascript
// File: migrations/deploys.js

const anchor = require("@project-serum/anchor");

module.exports = async function (provider) {
  anchor.setProvider(provider);

  // Add your deploy script here.
}
```

迁移是新功能，目前只支持这个简单的部署脚本。

## 新建程序New

```
anchor new <program-name>
```

在工作环境的 `programs/` 文件夹中创建新程序并用模板代码来初始化。

## 测试Test

```
anchor test
```

在配置好的集群运行集成测试，运行前会部署共环境中的所有程序。

如果配置的网络的是本地网络(localnetwork),测试前会自动启动本地网络，然后再运行测试。

::: 小贴士
要确保关掉其他的local validators, 否则`anchor test`会运行失败.

如果你想用本地validator来测试，可以运行 `anchor test --skip-local-validator`.
:::

当运行测试时，日志会被发送到 `.anchor/program-logs/<address>.<program-name>.log`

::: 小贴士
Anchor工作流[推荐](https://www.parity.io/paritys-checklist-for-secure-smart-contract-development/)
用非Rust语言来编写集成测试，来保证由对Rust语法误解造成的bug可以被测试覆盖到，而不被在测试中重复。
:::

## 升级Upgrade

```
anchor upgrade <target/deploy/program.so> --program-id <program-id>
```

用Solana的可升级BPF loader来升级链上程序代码.

## 验证Verify

```
anchor verify <program-id>
```

验证链上的字节码和本地的编译结果一致。