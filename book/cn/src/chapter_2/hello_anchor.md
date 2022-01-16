# Hello, Anchor!
要初始化一个项目，只需运行:

```
anchor init <new-workspace-name>
```

这会创建一个新的Anchor工作环境。这里是文件夹中的一些重要文件：

- `.anchor` 文件夹: 这里包括最新的程序日志和一个用来测试的本地账本
- `app` 文件夹: 如果你前端和合约放一个repo，你的frontend代码可以放到这个空文件夹里
- `programs` 文件夹: 合约代码放这里. 这里可以有多个程序文件，但一开始只有一个叫`<new-workspace-name>`的程序文件夹. 理由有一个叫`lib.rs`的样本代码文件.
- `tests` 文件夹: 这里有你的端到端测试. 已经包含`programs/<new-workspace-name>`的测试代码.
- `migrations` 文件夹: 这里可以放你的部署和迁移脚本.
- `Anchor.toml` 文件: 工作环境的配置文件. 在起始状态, 它包括：
    - 项目在localnet(`[programs.localnet]`)的地址
    - 一个你可以上传项目的(`[registry]`)
    - 一个你可以在测试用使用的provider (`[provider]`)
    - Anchor可以帮你执行的(`[scripts]`). `test` 脚本由 `anchor test`来触发运行. 你也可以通过`anchor run <script_name>`运行自己的脚本.