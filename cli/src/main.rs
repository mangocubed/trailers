use chrono::NaiveDate;
use clap::{Parser, Subcommand};

use trailers_core::graphql::{GraphqlSchema, GraphqlSchemaExt};
use trailers_core::jobs_storage;

#[derive(Parser)]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    GraphqlSchema,
    Populate {
        #[arg(short, long, required = false)]
        start_date: Option<NaiveDate>,
        #[arg(short, long, required = false)]
        end_date: Option<NaiveDate>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        CliCommand::GraphqlSchema => {
            let graphql_schema = GraphqlSchema::builder().finish();

            println!("{}", graphql_schema.sdl());
        }
        CliCommand::Populate { start_date, end_date } => {
            println!("Pushing job to populate database...");

            jobs_storage().await.push_populate(*start_date, *end_date).await;

            println!("Done!");
        }
    }
}
