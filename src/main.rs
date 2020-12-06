#[macro_use]
extern crate prettytable;
use prettytable::{Cell, Row, Table};
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

    // need to get the decimals so that we can output the right number.

    // output the real name if we can find it, otherwise, dump the raw pubkey
    // the format is as follows:
    // ?? can we something like tabular crate?
    // one line is one pool.
    // pool pub key | token-a(SRM) | balance-of-a | token-b(USDT) | balance-of-b |
    let mut table = Table::new();
    table.add_row(row![
        "Pool", "Token A", //"Balance A",
        "Token B",
        //"Balance B",
        //"Curve"
    ]);

    for pool in &pool_accounts {
        let mut cells = vec![];

        // pool address
        cells.push(Cell::new(format!("{}", pool.0).as_str()));

        // parse swap data
        let info = SwapInfo::unpack_from_slice(&pool.1.data).expect("invalid swap info");

        //println!("info: {:?}", &info);

        let token_a_pubkey = format!("{}", info.token_a_mint);
        let token_a_name = token_maps
            .get(&info.token_a_mint)
            .unwrap_or(&token_a_pubkey);
        cells.push(Cell::new(token_a_name));

        let token_b_pubkey = format!("{}", info.token_b_mint);
        let token_b_name = token_maps
            .get(&info.token_b_mint)
            .unwrap_or(&token_b_pubkey);
        cells.push(Cell::new(token_b_name));

        table.add_row(Row::new(cells));
    }

    table.printstd();

    // then we can enter an interactive mode where we can send out the swap instruction, that'll be quite cool.
}
