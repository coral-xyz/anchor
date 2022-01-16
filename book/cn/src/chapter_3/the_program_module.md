# Program模块

Program模块是我们定义业务逻辑的地方. 具体来说我们通过定义客户端可以调用的function来实现. 比如上一章中看到的`set_data` function。

```rust,ignore
#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: u64) -> ProgramResult {
        if ctx.accounts.token_account.amount > 0 {
            ctx.accounts.my_account.data = data;
        }
        Ok(())
    }
}
```

## Context参数

> [Context文档](https://docs.rs/anchor-lang/latest/anchor_lang/context/index.html)

每个接口function都会接受`Context`类型作为第一个参数. 通过context参数，我们可以拿到accounts (`ctx.accounts`), 执行程序的program id (`ctx.program_id`), 还有余下的accounts (`ctx.remaining_accounts`). `remaining_accounts` 是一个vector类型，它包含所有通过instruction传入，但没有定义在`Accounts` struct中的account. 这个方法可以处理account的数量为变量的情况, 例如.在不清楚有多少个玩家参与的情况下初始化一个游戏.

## Instruction Data

如果你的function需要instruction中传入额外的参数, 你只需要在context参数后面，多加参数就行. Anchor会自动帮你把instruction data反序列化为参数. 参数的数量没有限制. 你也可以传入自己定义的类型，但你需要使用`#[derive(AnchorDeserialize)]`宏属性，或者自己实现`AnchorDeserialize`. 这里是一个使用自定义类型作为参数的例子(`Data`):

```rust,ignore
...

#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: Data) -> ProgramResult {
        ctx.accounts.my_account.data = init_data.data;
        ctx.accounts.my_account.age = init_data.age;
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MyAccount {
    pub data: u64,
    pub age: u8
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Data {
    pub data: u64,
    pub age: u8
}

...
```

很方便的, `#[account]`为`MyAccount`实现了`Anchor(De)Serialize`,所以上面的例子也可以继续简化.

```rust,ignore
...

#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: MyAccount) -> ProgramResult {
        ctx.accounts.my_account.set_inner(data);
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MyAccount {
    pub data: u64,
    pub age: u8
}

...
```