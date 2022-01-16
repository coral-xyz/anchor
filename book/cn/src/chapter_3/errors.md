# Errors错误处理
Anchor程序中定义有三类错误. Anchor Internal Errors即Anchor内部错误, Custom Errors自定义错误, 和非Anchor错误.

Anchor代码生成的客户端可以自动解析Anchor Internal Errors还有Custom Errors，这样就可以展示相应的Error Code和报错信息。但是对非Anchor错误做不到这一点，这是就会把底层solana客户端库的返回原始raw error返回。

> (最终, 所有程序都返回同样的Error: [`ProgramError`](https://docs.rs/solana-program/latest/solana_program/program_error/enum.ProgramError.html). 这个Error有一个自定义的错误编号数字字段. 这就是Anchor存内部错误和自定义错误的error codes的地方. 客户端会先读取这个字段，然后读取IDL(通过这个把error code 映射到他们的报错信息) 来显示报错信息(Anchor内部错误error number=>message的映射是硬编码在客户端的). 这意味着不支持显示动态生成的报错信息，因为所有报错信息都因编码在客户端了。很快anchor会用日志logs来改变仅仅通过返回错误码来返回错误。这些日志也可以从客户端读取，并且支持动态错误信息。)

## Anchor内部错误

> [Anchor内部Error Code 文档](https://docs.rs/anchor-lang/latest/anchor_lang/__private/enum.ErrorCode.html)

Anchor有很多不同的内部error codes。这些并不是面向用户的，但是通过文档了解这些报错信息和他们的错误码，以及报错的原因还是很有好处的。例如，有些错误会在一个限制条件未被满足的时候被触发。例如一个被标记为`mut`的账户但`is_writable`熟悉却是`false`。

## Custom Errors自定义错误

你可以通过error attribute(`#[error]`)来自定义你程序独有的错误信息。只需要把它加到一个你自定义的enum上，然后你就可以把enum作为报错信息在程序中使用了。在此基础上，你还可以加一个message属性在enum的变型上，然后客户端就可以显示对应的报错信息了。自定义错误信息Custom Error的错误码从[自定义错误offset开始](https://docs.rs/anchor-lang/latest/anchor_lang/__private/constant.ERROR_CODE_OFFSET.html).

```rust,ignore
#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: MyAccount) -> ProgramResult {
        if data.data >= 100 {
            return Err(MyError::DataTooLarge.into());    
        }
        ctx.accounts.my_account.set_inner(data);
        Ok(())
    }
}


#[error]
pub enum MyError {
    #[msg("MyAccount may only hold data below 100")]
    DataTooLarge
}
```

你可以用[`require`](https://docs.rs/anchor-lang/latest/anchor_lang/macro.require.html) 宏来简化errors的处理. 上面的代码可以这样来简化 (注意 `>=` 改成了 `<`):
```rust,ignore
#[program]
mod hello_anchor {
    use super::*;
    pub fn set_data(ctx: Context<SetData>, data: MyAccount) -> ProgramResult {
        require!(data.data < 100, MyError::DataTooLarge); 
        ctx.accounts.my_account.set_inner(data);
        Ok(())
    }
}


#[error]
pub enum MyError {
    #[msg("MyAccount may only hold data below 100")]
    DataTooLarge
}
```
