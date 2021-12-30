extern crate proc_macro;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

/// Implements an [`Accounts`](./trait.Accounts.html) deserializer on the given
/// struct, applying any constraints specified via inert `#[account(..)]`
/// attributes upon deserialization.
///
/// # Example
///
/// ```ignore
/// #[derive(Accounts)]
/// pub struct Auth<'info> {
///     #[account(mut, has_one = authority)]
///     pub data: Account<'info, MyData>,
///     pub authority: Signer<'info>,
/// }
///
/// #[account]
/// pub struct MyData {
///   authority: Pubkey,
///   protected_data: u64,
/// }
/// ```
///
/// Here, any instance of the `Auth` struct created via
/// [`try_accounts`](trait.Accounts.html#tymethod.try_accounts) is guaranteed
/// to have been both
///
/// * Signed by `authority`.
/// * Checked that `&data.authority == authority.key`.
///
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
/// #[account(signer @ MyError::PayerSignatureMissing)]
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
/// #[account(mut @ MyError::DataAccountTwoNotMutable)]
/// pub data_account_two: Account<'info, MyData>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(has_one = &lt;target&gt;)]</code> <br><br><code>#[account(has_one = &lt;target&gt; @ &lt;custom_error&gt;)]</code>
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
///                 <code>#[account(address = &lt;expr&gt;)]</code> <br><br><code>#[account(address = &lt;expr&gt; @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Checks the account key matches the pubkey.<br>
///                 Custom errors are supported via <code>@</code>.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(address = crate::ID)]
/// pub data: Account<'info, MyData>,
/// #[account(address = crate::ID @ MyError::DataTwoInvalidAddress)]
/// pub data_two: Account<'info, MyData>
///                 </code></pre>
///             </td>
///         </tr>
///         <tr>
///             <td>
///                 <code>#[account(owner = &lt;expr&gt;)]</code> <br><br><code>#[account(owner = &lt;expr&gt; @ &lt;custom_error&gt;)]</code>
///             </td>
///             <td>
///                 Checks the account owner matches <code>expr</code>.<br>
///                 Custom errors are supported via <code>@</code>.<br><br>
///                 Example:
///                 <pre><code>
/// #[account(owner = Token::ID @ MyError::NotOwnedByTokenProgram)]
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
///     </tbody>
/// </table>
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
