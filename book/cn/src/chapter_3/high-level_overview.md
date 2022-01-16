# 总结概述
一个Anchor程序包括三部分. `program`模块, 带有`#[derive(Accounts)]`标记的Accounts structs, 还有`declareId`宏. `program`模块是我们编写业务逻辑的地方. Accounts structs是我们用来validate accounts的. `declareId`宏则创建了`ID`字段来储存程序的地址.

当你打开一个崭新的Anchor项目, 你会看到如下的模版代码:
```rust,ignore
// use this import to gain access to common anchor features
use anchor_lang::prelude::*;

// declare an id for your program
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// write your business logic here
#[program]
mod hello_anchor {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }
}

// validate incoming accounts here
#[derive(Accounts)]
pub struct Initialize {}
```
在接下来的章节中，我们会详细介绍。但暂时看这段代码，注意到一个接口是怎么通过`ctx`参数关联到其对应的Accounts struct了么？参数的类型是`Context`，里面泛型包着一个Accounts struct，也就是说，这就是我们实现Accounts验证的struct，也就是这个例子里面的`Initialize`。