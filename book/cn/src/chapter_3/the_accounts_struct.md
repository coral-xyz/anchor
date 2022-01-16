# Accounts Struct
Accounts struct 是我们定义传给instruction的accounts的数据结构，并且也用来定义这些accounts需要满足的限制条件. 我们主要通过两种方法来实现: 类型和限制条件. 

## Types类型

> [Account Types 类型的文档](https://docs.rs/anchor-lang/latest/anchor_lang/accounts/index.html)

每个类型都是为了针对一个特定的问题. 详细的解释可以参考[文档](https://docs.rs/anchor-lang/latest/anchor_lang/accounts/index.html). 这里我们只介绍最重要的类, `Account`类型.

### Account 类型

> [Account Reference](https://docs.rs/anchor-lang/latest/anchor_lang/accounts/account/struct.Account.html)

`Account`类型可以用来处理instruction中涉及到account所包含的，再反序列化之后的数据. 例如，接下来这个例子里，我们想再account中写一些数据:

```rust,ignore
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: u64) -> ProgramResult {
        ctx.accounts.my_account.data = data;
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct MyAccount {
    data: u64
}

#[derive(Accounts)]
pub struct SetData<'info> {
    #[account(mut)]
    pub my_account: Account<'info, MyAccount>
}
```

`Account`是围绕`T`的泛型. `T`是一个我们可以自己定义的类型. 在这个例子里,我们创建了`MyAccount` struct，它只有一个`data` 字段在储存一个`u64`. Account 要求 `T` 实现一些特定的functions (例如用来序列化和反序列化`T`的functions). 通常, 你可以用`#[account]`宏来为你的类型配置这些functions，如例子中所示。

最重要的一点是, `#[account]`宏会把账户的owner设置为它所在crate的`ID` (由我们一开始的`declareId`命令所创建). Account类型会帮我们自动验证传入instruction的`AccountInfo`所对应的`owner`字段指向了正确的程序. 在这个例子里, `MyAccount` 是在同一个crate 中定义的，所有`Account`会验证`my_account`的owner字段和`declareId`中定义的一致.

#### 对非Anchor程序的accounts使用`Account<'a, T>` 

有些情况下，你需要你的程序和非Anchor程序交互。你仍然可以利用`Account`的好处，但是你需要写一个自定义wrapper类型来替代`#[account]`宏. 比如, Anchor为token程序的accounts提供的wrapper类型TokenAccount，就可以直接用在`Account`里面. 

```rust,ignore
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

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

#[account]
#[derive(Default)]
pub struct MyAccount {
    data: u64,
    mint: Pubkey
}

#[derive(Accounts)]
pub struct SetData<'info> {
    #[account(mut)]
    pub my_account: Account<'info, MyAccount>,
    #[account(
        constraint = my_account.mint == token_account.mint,
        has_one = owner
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub owner: Signer<'info>
}
```

在这个例子里，如果调用的caller有admin权限，我们就可以写入account的`data`字段。我们通过检验Caller是不是拥有admin token 来检验它是不是有admin权限，具体说想要更新MyAccount的mint要吻合所有的token的mint。而这些检验都是由“constraints限制条件”这个功能实现的。我们会在下一节详细介绍。
这里的重点是我们用`TokenAccount`类来包裹在`Account`struct上来提供需要的functions，这样Anchor就可以确定通过instruction传入的account的确是属于token program的，并且自动反序列化。
这也意味着我们可以在我们的限制条件constraints和instruction中使用`TokenAccount`的properties(例如. `token_account.mint`)。

更多内容可以参考[reference for the Account type](https://docs.rs/anchor-lang/latest/anchor_lang/struct.Account.html)，来学习具体如何实现你自己的wrapper类来应付非Anchor程序的account。

## Constraints限制条件

> [Constraints reference](https://docs.rs/anchor-lang/latest/anchor_lang/derive.Accounts.html)

Account类型可以帮我们做很多，但是不够动态，不足以应付一个安全程序所需要的各种验证。而这就是Constraints可以帮我们的地方。

我们可以用下面的格式给account添加constraints:
```rust,ignore
#[account(<constraints>)]
pub account: AccountType
```

有的constraints支持自定义错误类型Custom Errors(之后我们会详细介绍[errors](./errors.md)):
```rust,ignore
#[account(...,<constraint> @ MyError::MyErrorVariant, ...)]
pub account: AccountType
```

在上面的一些例子里, 我们用`mut` constraint来标记`my_account`为可更改. 我们用`has_one`来验证`token_account.owner == owner.key()`. 最终我们用`constraint`来验证任意的表达式; 在下面的这个例子中，我们验证`TokenAccount`和admin的mint是同一个.

```rust,ignore
#[derive(Accounts)]
pub struct SetData<'info> {
    #[account(mut)]
    pub my_account: Account<'info, MyAccount>,
    #[account(
        constraint = my_account.mint == token_account.mint,
        has_one = owner
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub owner: Signer<'info>
}
```

所有constraints的细节都在文档里. 在核心内容的里程碑项目中，我们会介绍最重要的一些constraint。