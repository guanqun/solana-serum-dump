#[macro_use]
extern crate prettytable;
use prettytable::Table;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_token_swap::{solana_program::program_pack::Pack, state::SwapInfo};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

fn main() {
    let client = RpcClient::new_with_timeout(
        "https://api.mainnet-beta.solana.com".into(),
        Duration::from_secs(5),
    );

    let serum_program_id = Pubkey::from_str("9qvG1zUp8xF1Bi4m6UdRNby1BAAuaDrUxSpv4CmRRMjL")
        .expect("valid serum program id");
    let pool_accounts = client
        .get_program_accounts(&serum_program_id)
        .expect("can't get swap infos.");

    // it's not possible to get back the corresponding token names from on-chain data,
    // we'll need to consult it from third-party crate. I wrote a `spl-token-names` crate which does exactly this.
    let token_maps: HashMap<Pubkey, String> = spl_token_names::TOKENS
        .iter()
        .map(|info| {
            let pubkey = Pubkey::from_str(info.mint_address).expect("invalid pubkey");
            let name = String::from(info.token_symbol);
            (pubkey, name)
        })
        .collect();

    let mut table = Table::new();
    table.add_row(row![
        "Pool",
        "Token A",
        "Balance A",
        "Token B",
        "Balance B",
        "1 A ~ ? B"
    ]);

    for pool in &pool_accounts {
        // pool address
        let pool_address = format!("{}", pool.0);

        // parse swap data
        let info = SwapInfo::unpack_from_slice(&pool.1.data).expect("invalid swap info");

        let calculate_fn = |token: &Pubkey, mint: &Pubkey| {
            let accounts = client
                .get_multiple_accounts(&[*token, *mint])
                .expect("failed to get accounts");

            let token_data = accounts[0]
                .as_ref()
                .expect("failed to get token")
                .data
                .clone();
            let token_state = spl_token::state::Account::unpack(&token_data)
                .expect("failed to unpack token data");

            let mint_data = accounts[1]
                .as_ref()
                .expect("failed to get mint")
                .data
                .clone();
            let mint_state =
                spl_token::state::Mint::unpack(&mint_data).expect("failed to unpack mint data");

            let name = token_maps
                .get(mint)
                .map(|n| n.clone())
                .unwrap_or_else(|| format!("{}", token));

            let mut divisor: u64 = 1;
            for _ in 0..mint_state.decimals {
                divisor *= 10;
            }
            let balance = fraction::division::divide_to_string(
                token_state.amount,
                divisor,
                mint_state.decimals as usize,
                false,
            )
            .expect("failed to divide");

            (name, balance)
        };

        let (token_a_name, token_a_balance) = calculate_fn(&info.token_a, &info.token_a_mint);
        let (token_b_name, token_b_balance) = calculate_fn(&info.token_b, &info.token_b_mint);

        let ratio: f64 = f64::from_str(&token_b_balance).expect("invalid float")
            / f64::from_str(&token_a_balance).expect("invalid float");

        table.add_row(row![
            &pool_address,
            &token_a_name,
            &token_a_balance,
            &token_b_name,
            &token_b_balance,
            &format!("{:?}", ratio)
        ]);
    }

    table.printstd();
}
