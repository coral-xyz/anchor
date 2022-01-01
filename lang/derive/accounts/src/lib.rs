extern crate proc_macro;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

/// Implements an [`Accounts`](./trait.Accounts.html) deserializer on the given
/// struct. Can provide further functionality through the use of attributes.
///
/// # Table of Contents
/// - [Instruction Attribute](#instruction-attribute)
/// - [Constraints](#constraints)
///
/// # Instruction Attribute
///
/// You can access the instruction's arguments with the
/// `#[instruction(..)]` attribute. You have to list them
/// in the same order as in the instruction but you can
/// omit all arguments after the last one you need.
///
/// # Example
///
/// ```ignore
/// ...
/// pub fn initialize(ctx: Context<Create>, bump: u8, authority: Pubkey, data: u64) -> ProgramResult {
///     ...
///     Ok(())
/// }
/// ...
/// #[derive(Accounts)]
/// #[instruction(bump: u8)]
/// pub struct Initialize<'info> {
///     ...
/// }
/// ```
///
/// # Constraints
///
/// There are different types of constraints that can be applied with the `#[account(..)]` attribute.
///
/// - [Normal Constraints](#normal-constraints)
/// - [SPL Constraints](#spl-constraints)
/// # Normal Constraints
/// <table>
///     <thead>
///         <tr>
///             <th>Attribute</th>
///             <th>Description</th>
///         </tr>
///     </thead>
///     <tbody>
///         <tr>
///             <td>
///                 <code>#[account(signer)]</code> <br><br><code>#[account(signer @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Checks the given account signed the transaction.<br>
///                 Custom errors are supported via <code>@</code>.<br>
///                 Consider using the <code>Signer</code> type if you would only have this constraint on the account.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(signer)]
/// pub authority: AccountInfo<'info>,
/// #[account(signer @ MyError::MyErrorCode)]
/// pub payer: AccountInfo<'info>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(mut)]</code> <br><br><code>#[account(mut @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Checks the given account is mutable.<br>
///                 Makes anchor persist any state changes.<br>
///                 Custom errors are supported via <code>@</code>.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(mut)]
/// pub data_account: Account<'info, MyData>,
/// #[account(mut @ MyError::MyErrorCode)]
/// pub data_account_two: Account<'info, MyData>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(init, payer = &lt;target&gt;)]</code><br><br>
///                 <code>#[account(init, payer = &lt;target&gt;, space = &lt;num_bytes&gt;)]</code>
///             </td>
///             <td>
///                 Creates the account via a CPI to the system program and
///                 initializes it (sets its account discriminator).<br>
///                 Marks the account as mutable and is mutually exclusive with <code>mut</code>.<br>
///                 <ul>
///                     <li>
///                         Requires the <code>payer</code> constraint to also be on the account.
///                         The <code>payer</code> account pays for the
///                         account creation.
///                     </li>
///                     <li>
///                         Requires the system program to exist on the struct
///                         and be called <code>system_program</code>.
///                     </li>
///                     <li>
///                         Requires that the <code>space</code> constraint is specified
///                         or, if creating an <code>Account</code> type, the <code>T</code> of <code>Account</code>
///                         to implement the rust std <code>Default</code> trait.<br>
///                         When using the <code>space</code> constraint, one must remember to add 8 to it
///                         which is the size of the account discriminator.<br>
///                         The given number is the size of the account in bytes, so accounts that hold
///                         a variable number of items such as a <code>Vec</code> should use the <code>space</code>
///                         constraint instead of using the <code>Default</code> trait and allocate sufficient space for all items that may
///                         be added to the data structure because account size is fixed. Check out the <a href = "https://borsh.io/" target = "_blank" rel = "noopener noreferrer">borsh library</a>
///                         (which anchor uses under the hood for serialization) specification to learn how much
///                         space different data structures require.
///                     </li>
///                 <br>
///                 Example:
///                 <pre>
/// #[account]
/// #[derive(Default)]
/// pub struct MyData {
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data: u64
/// }&#10;
/// #[account]
/// pub struct OtherData {
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data: u64
/// }&#10;
/// #[derive(Accounts)]
/// pub struct Initialize<'info> {
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(init, payer = payer)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data_account: Account<'info, MyData>,
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(init, payer = payer, space = 8 + 8)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data_account_two: Account<'info, OtherData>,
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(mut)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub payer: Signer<'info>,
/// &nbsp;&nbsp;&nbsp;&nbsp;pub system_program: Program<'info, System>,
/// }
///                 </pre>
///                 </ul>
///                 <code>init</code> can be combined with other constraints (at the same time):
///                 <ul>
///                     <li>
///                         By default <code>init</code> sets the owner field of the created account to the
///                         currently executing program. Add the <code>owner</code> constraint to specify a
///                         different program owner.
///                     </li>
///                     <li>
///                         Use the <code>seeds</code> constraint together with <code>bump</code>to create PDAs.<br>
///                         <code>init</code> uses <code>find_program_address</code> to calculate the pda so the
///                         bump value can be left empty.<br>
///                         However, if you want to use the bump in your instruction,
///                         you can pass it in as instruction data and set the bump value like shown in the example,
///                         using the <code>instruction_data</code> attribute.
///                         Anchor will then check that the bump returned by <code>find_program_address</code> equals
///                         the bump in the instruction data.
///                     </li>
///                 </ul>
///                 Example:
///                 <pre>
/// #[derive(Accounts)]
/// #[instruction(bump: u8)]
/// pub struct Initialize<'info> {
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(init, payer = payer, seeds = [b"example_seed".as_ref()], bump = bump)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub pda_data_account: Account<'info, MyData>,
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(init, payer = payer, space = 8 + 8, owner = other_program.key())]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub account_for_other_program: AccountInfo<'info>,
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(init, payer = payer, space = 8 + 8, owner = other_program.key(), seeds = [b"other_seed".as_ref()], bump)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub pda_for_other_program: AccountInfo<'info>,
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(mut)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub payer: Signer<'info>,
/// &nbsp;&nbsp;&nbsp;&nbsp;pub system_program: Program<'info, System>,
/// &nbsp;&nbsp;&nbsp;&nbsp;pub other_program: Program<'info, OtherProgram>
/// }
///                 </pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(init_if_needed, payer = &lt;target&gt;)]</code><br><br>
///                 <code>#[account(init_if_needed, payer = &lt;target&gt;, space = &lt;num_bytes&gt;)]</code>
///             </td>
///             <td>
///                 Exact same functionality as the <code>init</code> constraint but only runs if the account does not exist yet.<br>
///                 If it does exist, it still checks whether the given init constraints are correct,
///                 e.g. that the account has the expected amount of space and, if it's a PDA, the correct seeds etc.
///                 <br><br>
///                 Example:
///                 <pre>
/// #[account]
/// #[derive(Default)]
/// pub struct MyData {
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data: u64
/// }&#10;
/// #[account]
/// pub struct OtherData {
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data: u64
/// }&#10;
/// #[derive(Accounts)]
/// pub struct Initialize<'info> {
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(init_if_needed, payer = payer)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data_account: Account<'info, MyData>,
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(init_if_needed, payer = payer, space = 8 + 8)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub data_account_two: Account<'info, OtherData>,
/// &nbsp;&nbsp;&nbsp;&nbsp;#[account(mut)]
/// &nbsp;&nbsp;&nbsp;&nbsp;pub payer: Signer<'info>,
/// &nbsp;&nbsp;&nbsp;&nbsp;pub system_program: Program<'info, System>
/// }
///                 </pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(seeds = &lt;seeds&gt;, bump)]</code><br><br>
///                 <code>#[account(seeds = &lt;seeds&gt;, bump = &lt;expr&gt;)]</code>
///             </td>
///             <td>
///                 Checks that given account is a PDA derived from the currently executing program,
///                 the seeds, and if provided, the bump. If not provided, anchor uses the canonical
///                 bump.
///                 <br><br>
///                 Example:
///                 <pre><code>
/// #[account(seeds = [b"example_seed], bump)]
/// pub canonical_pda: AccountInfo<'info>,
/// #[account(seeds = [b"other_seed], bump = 142)]
/// pub arbitrary_pda: AccountInfo<'info>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(has_one = &lt;target&gt;)]</code><br><br>
///                 <code>#[account(has_one = &lt;target&gt; @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Checks the <code>target</code> field on the account matches the
///                 key of the <code>target</code> field in the Accounts struct.<br>
///                 Custom errors are supported via <code>@</code>.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(mut, has_one = authority)]
/// pub data: Account<'info, MyData>,
/// pub authority: Signer<'info>
///                 </code></pre>
///                 In this example <code>has_one</code> checks that <code>data.authority = authority.key()</code>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(address = &lt;expr&gt;)]</code><br><br>
///                 <code>#[account(address = &lt;expr&gt; @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Checks the account key matches the pubkey.<br>
///                 Custom errors are supported via <code>@</code>.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(address = crate::ID)]
/// pub data: Account<'info, MyData>,
/// #[account(address = crate::ID @ MyError::MyErrorCode)]
/// pub data_two: Account<'info, MyData>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(owner = &lt;expr&gt;)]</code><br><br>
///                 <code>#[account(owner = &lt;expr&gt; @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Checks the account owner matches <code>expr</code>.<br>
///                 Custom errors are supported via <code>@</code>.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(owner = Token::ID @ MyError::MyErrorCode)]
/// pub data: Account<'info, MyData>,
/// #[account(owner = token_program.key())]
/// pub data_two: Account<'info, MyData>,
/// pub token_program: Program<'info, Token>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(executable)]</code>
///             </td>
///             <td>
///                 Checks the account is executable (i.e. the account is a program).<br>
///                 You may want to use the <code>Program</code> type instead.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(executable)]
/// pub my_program: AccountInfo<'info>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(rent_exempt = skip)]</code><br><br>
///                 <code>#[account(rent_exempt = enforce)]</code>
///             </td>
///             <td>
///                 Enforces rent exemption with <code>= enforce</code>.<br>
///                 Skips rent exemption check that would normally be done
///                 through other constraints with <code>= skip</code>,
///                 e.g. when used with the <code>zero</code> constraint<br><br>
///                 Example:
///                 <pre><code>
/// #[account(zero, rent_exempt = skip)]
/// pub skipped_account: Account<'info, MyData>,
/// #[account(rent_exempt = enforce)]
/// pub enforced_account: AccountInfo<'info>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(zero)]</code>
///             </td>
///             <td>
///                 Checks the account discriminator is zero.<br>
///                 Enforces rent exemption unless skipped with <code>rent_exempt = skip</code><br><br>
///                 Example:
///                 <pre><code>
/// #[account(zero)]
/// pub my_account: Account<'info, MyData>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(close = &lt;target&gt;)]</code>
///             </td>
///             <td>
///                 Marks the account as closed at the end of the instruction’s execution
///                 (sets its discriminator to the <code>CLOSED_ACCOUNT_DISCRIMINATOR</code>)
///                 and sends its lamports to the specified account.
///                 <br><br>
///                 Example:
///                 <pre><code>
/// #[account(close = receiver)]
/// pub data_account: Account<'info, MyData>,
/// #[account(mut)]
/// pub receiver: SystemAccount<'info>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(constraint = &lt;expr&gt;)]</code><br><br><code>#[account(constraint = &lt;expr&gt; @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Constraint that checks whether the given expression evaluates to true.<br>
///                 Use this when no other constraint fits your use case.
///                 <br><br>
///                 Example:
///                 <pre><code>
/// #[account(constraint = data_account.keys[0].age == other_data_account.market.apple.age)]
/// pub data_account: Account<'info, MyData>,
/// pub other_data_account: Account<'info, OtherData>
///                 </code></pre>
///             </td>
///         </tr>
///     </tbody>
/// </table>
///
/// # SPL Constraints
///
/// The full list of available attributes is as follows.
///
/// | Attribute | Location | Description |
/// |:--|:--|:--|
/// | `#[account(signer)]`<br><br>`#[account(signer @ <custom_error>)]` | On raw `AccountInfo` structs. | Checks the given account signed the transaction. Custom errors are supported via `@`. |
/// | `#[account(mut)]`<br><br>`#[account(mut @ <custom_error>)]` | On `AccountInfo`, `Account` or `CpiAccount` structs. | Marks the account as mutable and persists the state transition. Custom errors are supported via `@`. |
/// | `#[account(init)]` | On `Account` structs. | Marks the account as being initialized, creating the account via the system program. |
/// | `#[account(init_if_needed)]` | On `Account` structs. | Same as `init` but skip if already initialized. |
/// | `#[account(zero)]` | On `Account` structs. | Asserts the account discriminator is zero. |
/// | `#[account(close = <target>)]` | On `Account` and `AccountLoader` structs. | Marks the account as being closed at the end of the instruction's execution, sending the rent exemption lamports to the specified <target>. |
/// | `#[account(has_one = <target>)]`<br><br>`#[account(has_one = <target> @ <custom_error>)]` | On `Account` or `CpiAccount` structs | Checks the `target` field on the account matches the `target` field in the struct deriving `Accounts`. Custom errors are supported via `@`. |
/// | `#[account(seeds = [<seeds>], bump? = <target>, payer? = <target>, space? = <target>, owner? = <target>)]` | On `AccountInfo` structs | Seeds for the program derived address an `AccountInfo` struct represents. If bump is provided, then appends it to the seeds. On initialization, validates the given bump is the bump provided by `Pubkey::find_program_address`.|
/// | `#[account(constraint = <expression>)]`<br><br>`#[account(constraint = <expression> @ <custom_error>)]` | On any type deriving `Accounts` | Executes the given code as a constraint. The expression should evaluate to a boolean. Custom errors are supported via `@`. |
/// | `#[account("<literal>")]` | Deprecated | Executes the given code literal as a constraint. The literal should evaluate to a boolean. |
/// | `#[account(rent_exempt = <skip>)]` | On `AccountInfo` or `Account` structs | Optional attribute to skip the rent exemption check. By default, all accounts marked with `#[account(init)]` will be rent exempt, and so this should rarely (if ever) be used. Similarly, omitting `= skip` will mark the account rent exempt. |
/// | `#[account(executable)]` | On `AccountInfo` structs | Checks the given account is an executable program. |
/// | `#[account(state = <target>)]` | On `CpiState` structs | Checks the given state is the canonical state account for the target program. |
/// | `#[account(owner = <target>)]`<br><br>`#[account(owner = <target> @ <custom_error>)]` | On `CpiState`, `CpiAccount`, and `AccountInfo` | Checks the account owner matches the target. Custom errors are supported via `@`. |
/// | `#[account(address = <pubkey>)]`<br><br>`#[account(address = <pubkey> @ <custom_error>)]` | On `AccountInfo` and `Account` | Checks the account key matches the pubkey. Custom errors are supported via `@`. |
// TODO: How do we make the markdown render correctly without putting everything
//       on absurdly long lines?
#[proc_macro_derive(Accounts, attributes(account, instruction))]
pub fn derive_anchor_deserialize(item: TokenStream) -> TokenStream {
    parse_macro_input!(item as anchor_syn::AccountsStruct)
        .to_token_stream()
        .into()
}
