mod avail;

use std::env;
use std::time::SystemTime;
use anyhow::bail;
use jsonrpsee::core::__reexports::tokio;
use sqlx::{postgres::PgPoolOptions, Row};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        panic!("Incorrect number of arguments, expected 2, got {}", args.len());
    }

    let first_batch = args[1].parse::<i32>()?;
    let db_url = &args[2];

    let pool = PgPoolOptions::new()
        .connect(db_url)
        .await?;

    let app_id = 17;
    let api_url = "wss://turing-rpc.avail.so/ws".to_string();
    let seed = "";
    let client = avail::AvailClient::new(app_id, api_url, seed).await?;

    let rows = sqlx::query("SELECT l1_batch_number, pubdata_input
            FROM data_availability
                INNER JOIN l1_batches on data_availability.l1_batch_number = l1_batches.number
            WHERE l1_batch_number > $1 AND blob_id = ''
            ORDER BY l1_batch_number")
        .bind(first_batch )
        .fetch_all(&pool)
        .await?;

    let mut i = 0;
    let total = rows.len();
    for row in rows {
        i += 1;
        let l1_batch_number: i64 = row.get("l1_batch_number");
        let pubdata_input: Vec<u8> = row.get("pubdata_input");
        println!("({} of {}) batch: {:?}, len pubdata: {:?}", i, total, l1_batch_number, pubdata_input.len());

        let dispatch_start_time = SystemTime::now();
        let blob_id = match client.dispatch_blob(l1_batch_number as u32, pubdata_input.clone()).await {
            Ok(res) => res,
            Err(e) => {
                println!("dispatch_blob error: {:?}, retrying...", e);
                client.dispatch_blob(l1_batch_number as u32, pubdata_input).await?
            }
        };
        let dispatch_took = dispatch_start_time.elapsed()?.as_secs();
        println!("dispatch took: {:?}", dispatch_took);
        println!("blob_id: {:?}", blob_id);

        sqlx::query("UPDATE data_availability
            SET blob_id = $1
            WHERE l1_batch_number = $2")
            .bind(blob_id)
            .bind(l1_batch_number )
            .execute(&pool)
            .await?;
    }

    Ok(())
}
