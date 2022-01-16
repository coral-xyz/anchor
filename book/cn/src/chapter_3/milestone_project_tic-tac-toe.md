# 里程碑项目 - Tic-Tac-Toe三连棋游戏
你现在已经做好准备开发你的第一个Anchor项目了。创建一个新的Anchor工作环境：

```
anchor init tic-tac-toe
```

这个程序会有两个指令(instructions). 首先，我们需要创建游戏(setup)，我们需要保存谁在玩，并且创建一个可以玩的棋盘。然后，每个选手轮流行动，直到一方胜出或者达到平局。

## 创建游戏Setting up the game

### 状态State

我们先开始思考一下我们应该储存哪。每局游戏有玩家，回合，游戏板，还有游戏状态。这个游戏状态描述游戏是不是还在进行，平局或者一方赢了。我们可以把所有这些状态都存在一个account里面。着意味着每新一局游戏都会有自己的account。把下面这段代码加到`lib.rs`的最底部：
```rust,ignore
#[account]
pub struct Game {
    players: [Pubkey; 2],          // 64
    turn: u8,                      // 1
    board: [[Option<Sign>; 3]; 3], // 9 * (1 + 1) = 18
    state: GameState,              // 32 + 1
}
```
这就是game account. 在字段定义边上, 你看一看到每个字段要求的字节数. 这在之后会用到. 让我们把`Sign`和`GameState`类型也加上。
```rust,ignore
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GameState {
    Active,
    Tie,
    Won { winner: Pubkey },
}

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    FromPrimitive,
    ToPrimitive,
    Copy,
    Clone,
    PartialEq,
    Eq
)]
pub enum Sign {
    X,
    O,
}
```

`GameState`和`Sign`都通过derive引入了一些traits. 其中`AnchorSerialize`和`AnchorDeserialize`最重要. 所有在被`#[account]`标记的类型内部面使用的类型必须实现这两个traits(或者本身也被标记了`#[account]`). 所有其他的traits也对我们的游戏逻辑很重要，我们会在之后用到。通常一个好的习惯是使用更多的traits来让程序对外部的接口用户更友好(参考 [Rust的API指南](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits)) 但是处于简洁考虑, 在这个指南里我们就不详细介绍了。

暂时这还不能运行，因为`FromPrimitive`和`ToPrimitive`对程序是未知的。 在`src`外面的`Cargo.toml`文件  (不是工作环境跟目的)然后加上这两个依赖:
```toml
num-traits = "0.2"
num-derive = "0.3"
```
接下来, 在`lib.rs`最上面import:
```rust,ignore
use num_derive::*;
use num_traits::*;
```

### Setup Instruction命令

在我们实现游戏逻辑之前, 我们可以先加上在初始状态创建游戏的instruction。把现有的instruction function还有accounts struct为`setup_game`和`SetupGame`。 现在我们来考虑一下创建游戏需要哪些accounts。显然，我们需要game account。在对它进行各种赋值之前，我们要先创建它。因此我们要用`init`限制条件。
```rust,ignore
#[derive(Accounts)]
pub struct SetupGame<'info> {
    #[account(init)]
    pub game: Account<'info, Game>
}
```
`init`应该会立刻报错并且要添加一个payer. 为什么需要payer呢? 因为`init`会创建一个`免租`(`rent-exempt`) accounts因此必须有人要付出足够的费用. 自然的, 如果我们想用某人的钱, 我们肯定需要他签名，并且把他的账号标记为可更改。
```rust,ignore
#[derive(Accounts)]
pub struct SetupGame<'info> {
    #[account(init, payer = player_one)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub player_one: Signer<'info>
}
```
`init`可能还是有报错. 它需要system program也在struct里面，因为`init`创建game account，需要对system program调用。 那我们把它也加上。
```rust,ignore
#[derive(Accounts)]
pub struct SetupGame<'info> {
    #[account(init, payer = player_one)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub player_one: Signer<'info>,
    pub system_program: Program<'info, System>
}
```
完成`SetupGame`还有最后一步。每个account都是按照固定的空间大小创建的。 `init`可以估计一个account所需要的空间，如果它由`Default`导出。因为，让我们来给`Game`实现一下`Default`。
```rust,ignore
#[account]
#[derive(Default)] <-- add this
pub struct Game {...
```
 and `GameState`.
```rust,ignore
impl Default for GameState {
    fn default() -> Self {
        Self::Active
    }
}
```

这样, `SetupGame`就完成了，我们可以继续`setup_game` function的开发。(如果你想做侦探，你可以试着检查，为什么目前这个实现依然有问题。 提示: 查看[borsh 文档](https://borsh.io/)，也就是Anchor用的序列化库。如果你看不出来问题，也没关系。 我们很快就会介绍它)

让我们先给`setup_game`function加个参数.
```rust,ignore
pub fn setup_game(ctx: Context<SetupGame>, player_two: Pubkey) -> ProgramResult {
    Ok(())
}
```
为什么不直接把`player_two`作为account加到accounts struct里面呢？有两个原因，首先，加到accounts里面会让交易占用更多的空间因为需要记录account是不是可写入(writable)和是不是签名者(signer)。但是我们并不需要这两个状态，只需要地址address。第二个也是更重要一个原因，如果用到了同一个账号，一个交易可以影响到网络中同时在处理的其他交易。例如，如果我们把`player_two`加到了accounts struct, 那再处理我们的交易的时候，任何其他交易都不可以编辑`player_two`的account。 也就是说, 我们阻塞了所有其他的想编辑`player_two`的account的交易, 即使我们根本不想动这个account. 我们只在乎它的地址！

完成instruction function，设置game的初始值:
```rust,ignore
pub fn setup_game(ctx: Context<SetupGame>, player_two: Pubkey) -> ProgramResult {
    let game = &mut ctx.accounts.game;
    game.players = [ctx.accounts.player_one.key(), player_two];
    game.turn = 1;
    Ok(())
}
```

现在，运行`anchor build`。除了编译你的程序，这个命令还会为你的程序创建一个[IDL](https://en.wikipedia.org/wiki/Interface_description_language)。 具体位置在`target/idl`. Anchor的typescript客户端可以自动解析IDL，并根据它生成函数. 这意味着每个Anchor程序都有免费的Typescript客户端(自动生成)! (理论上说, 测试之前你不需要调用anchor build. `anchor test`会帮你调用.)

### 测试Setup Instruction

该测试代码了! 去项目根目录的`tests`文件夹. 打开`tic-tac-toe.ts`文件然后删掉`it`test. 然后, 把下面的带吧添加到`describe`部分:
```typescript
  it('setup game!', async() => {
    const gameKeypair = anchor.web3.Keypair.generate();
    const playerOne = program.provider.wallet;
    const playerTwo = anchor.web3.Keypair.generate();
    await program.rpc.setupGame(playerTwo.publicKey, {
      accounts: {
        game: gameKeypair.publicKey,
        playerOne: playerOne.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      },
      signers: [gameKeypair]
    });

    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(gameState.turn).to.equal(1);
    expect(gameState.players)
      .to
      .eql([playerOne.publicKey, playerTwo.publicKey]);
    expect(gameState.state).to.eql({ active: {} });
    expect(gameState.board)
      .to
      .eql([[null,null,null],[null,null,null],[null,null,null]]);
  });
```

然后把这个加到文件开头:
```typescript
import { expect } from 'chai';
```

测试开始我们先创建一些公私钥对。值得注意的是, `playerOne`并不是公私钥对，而是程序provider的钱包. Provider的细节是在项目根目录的`Anchor.toml`文件中定义的。
然后, 我们发送交易。因为anchor的typescript客户端解析了IDL,所有交易的输入都有类型。例如，如果你少加了一个account,typescript就会报错。 
接下来我们介绍transaction function的结构. 第一是instruction的参数。 就这个function来说, 是second player的公钥。接下来是accounts. 最后, 我们加上signers数组。 这里我们必须加上`gameKeypair`也就是game的公私钥对，因为每当一个account被创建, 它都要对创建账号的transaction签名。 我们不需要加`playerOne`，尽管它在程序中定义的是`Signer`类型，因为它是程序的provider并且本身就会对transaction签名。

在transaction返回结果后，我们可以读取game account的状态。通常你可以通过`program.account` namespace来读取account的状态。 
最后，我们来验证game已经被成功的创建了。Anchor的typescript客户端会这样反序列化rust的enum: `{ active: {}}`对应一个没有字段的变型，而`{ won: { winner: Pubkey }}` 对应有字段值的变型. `Option`的`None`变型会被转化为`null`.`Some(x)`会被转化为`x`反序列化的结果.

现在运行`anchor test`。这会启动(之后自动关闭)一个本地的validator(确认在着之前你自己没有运行validator)然后运行你定义在`Anchor.toml`中的测试脚本。

## 实现游戏Playing the game

### The Play Instruction

`Play` accounts struct很容易懂. 我们需要一个game account还有一个玩家player:
```rust,ignore
#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}
```

`player`需要签名否则，其他人也可以来冒名顶替.

接下来，我们实现游戏逻辑:

```rust,ignore
impl Game {
    pub fn is_active(&self) -> bool {
        self.state == GameState::Active
    }

    fn current_player_index(&self) -> usize {
        ((self.turn - 1) % 2) as usize
    }

    pub fn current_player(&self) -> Pubkey {
        self.players[self.current_player_index()]
    }

    pub fn play(&mut self, tile: &Tile) -> ProgramResult {
        if !self.is_active() {
            return Err(TicTacToeError::GameAlreadyOver.into());
        }
        match tile {
            tile
            @ Tile {
                row: 0..=2,
                column: 0..=2,
            } => match self.board[tile.row as usize][tile.column as usize] {
                Some(_) => return Err(TicTacToeError::TileAlreadySet.into()),
                None => {
                    self.board[tile.row as usize][tile.column as usize] =
                        Some(Sign::from_usize(self.current_player_index()).unwrap());
                }
            },
            _ => return Err(TicTacToeError::TileOutOfBounds.into()),
        }

        self.update_state();

        if let GameState::Active = self.state {
            self.turn += 1;
        }

        Ok(())
    }

    fn is_winning_trio(&self, trio: [(usize, usize); 3]) -> bool {
        let [first, second, third] = trio;
        self.board[first.0][first.1].is_some()
            && self.board[first.0][first.1] == self.board[second.0][second.1]
            && self.board[first.0][first.1] == self.board[third.0][third.1]
    }

    fn update_state(&mut self) {
        for i in 0..=2 {
            // three of the same in one row
            if self.is_winning_trio([(i, 0), (i, 1), (i, 2)]) {
                self.state = GameState::Won {
                    winner: self.current_player(),
                };
                return;
            }
            // three of the same in one column
            if self.is_winning_trio([(0, i), (1, i), (2, i)]) {
                self.state = GameState::Won {
                    winner: self.current_player(),
                };
                return;
            }
        }

        // three of the same in one diagonal
        if self.is_winning_trio([(0, 0), (1, 1), (2, 2)])
            || self.is_winning_trio([(0, 2), (1, 1), (2, 0)])
        {
            self.state = GameState::Won {
                winner: self.current_player(),
            };
            return;
        }

        // reaching this code means the game has not been won,
        // so if there are unfilled tiles left, it's still active
        for row in 0..=2 {
            for column in 0..=2 {
                if self.board[row][column].is_none() {
                    return;
                }
            }
        }

        // game has not been won
        // game has no more free tiles
        // -> game ends in a tie
        self.state = GameState::Tie;
    }
}
```

这段代码我们就不一起细看了，因为是比较简单的rust代码。毕竟三连棋本身就是个简单的游戏！大致上，每当`play`被调用，会有这些动作:
1. return error if 游戏结束 or
return error if 输入的行或者列的位置在3x3 board外面 or
return error if tile on 板子上的位置已经被填过了
2. 确定现在的player 并且把落棋子的位置设为 X 或 O
3. 更新 game state
4. if game 还是 active状态, 增加 the turn回合数

目前, 代码还是无法编译，因为我们需要实现`Tile`类型
```rust,ignore
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Tile {
    row: u8,
    column: u8,
}
```

and the `TicTacToeError` type.
```rust,ignore
#[error]
pub enum TicTacToeError {
    TileOutOfBounds,
    TileAlreadySet,
    GameAlreadyOver,
    NotPlayersTurn,
}
```

最后, 我们把 `play` function添加到program模块。
```rust,ignore
pub fn play(ctx: Context<Play>, tile: Tile) -> ProgramResult {
    let game = &mut ctx.accounts.game;

    require!(
        game.current_player() == ctx.accounts.player.key(),
        TicTacToeError::NotPlayersTurn
    );

    game.play(&tile)
}
```

在accounts struct中我们已经检查过`player` account已经对transaction签名，但是我们没有检查这个`player`是不是我们预期game中所对应的`player`。这就是`play`里面`require`的作用。

### Testing the Play Instruction

测试`play` instruction的方法和之前类似。 为了不写重复代码, 我们可以在测试文件的最上面创建一个helper function:
```typescript
async function play(program, game, player,
    tile, expectedTurn, expectedGameState, expectedBoard) {
  await program.rpc.play(tile, {
    accounts: {
      player: player.publicKey,
      game
    },
    signers: player instanceof (anchor.Wallet as any) ? [] : [player]
  });

  const gameState = await program.account.game.fetch(game);
  expect(gameState.turn).to.equal(expectedTurn);
  expect(gameState.state).to.eql(expectedGameState);
  expect(gameState.board)
    .to
    .eql(expectedBoard);
}
```

你可以创建一个新的`it`测试, 像上个测试里面那样创建game, 然后连续调用我们刚加的`play` function来模拟玩一整局游戏。 Let's begin with the first turn:
```typescript
it('player one wins', async() => {
    const gameKeypair = anchor.web3.Keypair.generate();
    const playerOne = program.provider.wallet;
    const playerTwo = anchor.web3.Keypair.generate();
    await program.rpc.setupGame(playerTwo.publicKey, {
      accounts: {
        game: gameKeypair.publicKey,
        playerOne: playerOne.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      },
      signers: [gameKeypair]
    });

    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(gameState.turn).to.equal(1);
    expect(gameState.players)
      .to
      .eql([playerOne.publicKey, playerTwo.publicKey]);
    expect(gameState.state).to.eql({ active: {} });
    expect(gameState.board)
      .to
      .eql([[null,null,null],[null,null,null],[null,null,null]]);

    await play(
      program,
      gameKeypair.publicKey,
      playerOne,
      {row: 0, column: 0},
      2,
      { active: {}, },
      [
        [{x:{}},null,null],
        [null,null,null],
        [null,null,null]
      ]
    );
});
```

然后再次运行`anchor test`，但你会发现一个报错:
```
Error: 3004: Failed to serialize the account
```
这咋办? 我们知道这肯定是运行play时候的问题, 因为`setupGame`的测试运行成功。
另外, 报错说`serialize`(序列化), 不是`deserialize`(反序列化). 所以我们的代码运行了，然后anchor试图保存数据，但出错了。
这类报错通常意味着我们的account太小了，无法保存所有的数据，而在这里恰恰就是这个问题。

我们再细看一下`Game` struct还有我们创建它的方法:
```rust,ignore
#[account]
#[derive(Default)]
pub struct Game {
    players: [Pubkey; 2],          // 64
    turn: u8,                      // 1
    board: [[Option<Sign>; 3]; 3], // 9 * (1 + 1) = 18
    state: GameState,              // 32 + 1
}

...
#[account(init, payer = player_one)]
pub game: Account<'info, Game>,
...

```

回忆我们使用`Default`标记`Game`，因为`init`会根据`Default`试图推到所需要的正确的空间大小, “试图”是关键词. 再我们不明确定义account所需要的空间(`space` requirement)，Anchor会调用account的`default`，然后把它用borsh序列库转化为一个vector类型. 然后Anchor会用这个vector的长度作为account所需要的空间大小。
那我们一步一步的用[borsh的文档](https://borsh.io/)来过一下我们的代码. 旁边的注释告诉我们必须的空间大小要求, 也就是，对应类型的最大所占空间。
- Pubkey as a vector has a length of `32` so `2*32 = 64` ✅
- u8 as a vector has a length of `1` so `1 = 1` ✅
- board's default (`9 * None`) as a vector has a length of `9 != 18` ❌
- state's default as a vector is `1 != 33` ❌

结论是`init`目前是预留了75字节给我们的account但account可能需要(64 + 1 + 18 + 33) = 116字节。
我们可以这样把这个数字加到我们的实现里面:
```rust,ignore
impl Game {
    const MAXIMUM_SIZE: usize = 116;

    ... // other functions
}
```

```rust,ignore
...
#[account(init, payer = player_one, space = Game::MAXIMUM_SIZE + 8)]
pub game: Account<'info, Game>,
...
```
再game状态的基础上, 我们还要额外加上`8`字节作为space. 这个额外的空间是留点给Anchor自动设置的`discriminator`的。 简短说,discriminator是Anchor用来区分同一个程序中不同account的标识。

> (那用`mem::size_of<Game>()`? 这个方法几乎可以，但还是不行。 问题是borsh库总是把option的标识序列化为1字节然后额外的x字节如果值是`Some`. Rust在可以的时候会用null-pointer优化来让Option的标识占零字节, 所有有的时候option和它的内容一样大。`Sign`就是被这样处理的。 这意味着`MAXIMUM_SIZE`可能会被表示为`mem::size_of<Game>() + 9`，这还是不对的)

再运行`anchor test`应该就可以成功了。 你可以自己完成余下的测试了。试着模拟一个平局和赢的局!

完善的测试还应该包括试图攻击合约的测试。你可以检查你时候妥善的考虑到了用非常规参数恶意调用你`play`的情况。 比如:
```typescript
try {
  await play(
    program,
    gameKeypair.publicKey,
    playerTwo,
    {row: 5, column: 1}, // out of bounds row
    4,
    { active: {}, },
    [
      [{x:{}},{x: {}},null],
      [{o:{}},null,null],
      [null,null,null]
    ]
  );
  // we use this to make sure we definitely throw an error
  chai.assert(false, "should've failed but didn't ");
} catch (error) {
  expect(error.code).to.equal(6000);
}
```

## 部署

Solana有三个网路: 主网`mainnet-beta`, 开发网`devnet`, 和测试网`testnet`.
对开发者来说, `devnet`和`mainnet-beta`最值得关注。 `devnet`是你最接近真实的测试环境，相比本地环境`localnet`。 `testnet`通常只和Validators相关。

我们将在`devnet`部署.

这里是你的部署检查清单 🚀

1. 运行 `anchor build`. 你的program keypair会保存于`target/deploy`. 记得这个需要保密. 你可以在其他的网络使用.
2. 运行 `solana address -k target/deploy/tic_tac_toe-keypair.json`然后复制地址到`lib.rs`顶部的`declare_id!`宏.
3. 运行 `anchor build` 多一次. 这步很必要，因为需要保证新的program id存入二进制里面。
4. 更改`Anchor.toml`中的`provider.cluster`变量为`devnet`.
5. 运行 `anchor deploy`
6. 运行 `anchor test`

有关部署的内容还有很多。例如，理解BPFLoader的工作原理,如何管理keys, 还有如何升级你的程序等等. 继续读更多文档来学习把!

干得漂亮! 你完成了核心内容. 现在可以去学习更高级的Anchor内容了。