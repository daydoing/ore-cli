mod balance;
mod busses;
mod claim;
mod cu_limits;
mod mine;
mod register;
mod rewards;
mod send_and_confirm;
mod treasury;
mod utils;

use std::sync::Arc;

use clap::{command, Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair};

struct Miner {
    pub private_key: String,
    pub priority_fee: u64,
    pub rpc_client: Arc<RpcClient>,
}

#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    #[arg(
        long,
        value_name = "NETWORK_URL",
        help = "Network address of your RPC provider",
        global = true
    )]
    rpc: Option<String>,

    #[arg(
        long,
        value_name = "PRIVATE_KEY",
        help = "Private key to use",
        global = true
    )]
    private_key: Option<String>,

    #[arg(
        long,
        value_name = "MICROLAMPORTS",
        help = "Number of microlamports to pay as priority fee per transaction",
        default_value = "0",
        global = true
    )]
    priority_fee: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Fetch the Ore balance of an account")]
    Balance(BalanceArgs),

    #[command(about = "Fetch the distributable rewards of the busses")]
    Busses(BussesArgs),

    #[command(about = "Mine Ore using local compute")]
    Mine(MineArgs),

    #[command(about = "Claim available mining rewards")]
    Claim(ClaimArgs),

    #[command(about = "Fetch your balance of unclaimed mining rewards")]
    Rewards(RewardsArgs),

    #[command(about = "Fetch the treasury account and balance")]
    Treasury(TreasuryArgs),
}

#[derive(Parser, Debug)]
struct BalanceArgs {
    #[arg(
        // long,
        value_name = "ADDRESS",
        help = "The address of the account to fetch the balance of"
    )]
    pub address: Option<String>,
}

#[derive(Parser, Debug)]
struct BussesArgs {}

#[derive(Parser, Debug)]
struct RewardsArgs {
    #[arg(
        // long,
        value_name = "ADDRESS",
        help = "The address of the account to fetch the rewards balance of"
    )]
    pub address: Option<String>,
}

#[derive(Parser, Debug)]
struct MineArgs {
    #[arg(
        long,
        short,
        value_name = "THREAD_COUNT",
        help = "The number of threads to dedicate to mining",
        default_value = "1"
    )]
    threads: u64,
}

#[derive(Parser, Debug)]
struct TreasuryArgs {}

#[derive(Parser, Debug)]
struct ClaimArgs {
    #[arg(
        // long,
        value_name = "AMOUNT",
        help = "The amount of rewards to claim. Defaults to max."
    )]
    amount: Option<f64>,

    #[arg(
        // long,
        value_name = "TOKEN_ACCOUNT_ADDRESS",
        help = "Token account to receive mining rewards."
    )]
    beneficiary: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize miner.
    let cluster = args.rpc.expect("");
    let private_key = args.private_key.expect("");
    let rpc_client = RpcClient::new_with_commitment(cluster, CommitmentConfig::confirmed());

    let miner = Arc::new(Miner::new(
        Arc::new(rpc_client),
        args.priority_fee,
        private_key,
    ));

    // Execute user command.
    match args.command {
        Commands::Balance(args) => {
            miner.balance(args.address).await;
        }
        Commands::Busses(_) => {
            miner.busses().await;
        }
        Commands::Rewards(args) => {
            miner.rewards(args.address).await;
        }
        Commands::Treasury(_) => {
            miner.treasury().await;
        }
        Commands::Mine(args) => {
            miner.mine(args.threads).await;
        }
        Commands::Claim(args) => {
            miner.claim(args.beneficiary, args.amount).await;
        }
    }
}

impl Miner {
    pub fn new(rpc_client: Arc<RpcClient>, priority_fee: u64, private_key: String) -> Self {
        Self {
            rpc_client,
            private_key,
            priority_fee,
        }
    }

    pub fn signer(&self) -> Keypair {
        Keypair::from_base58_string(&self.private_key.trim())
    }
}
