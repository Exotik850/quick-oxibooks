use quick_oxibooks::functions::reports::QBReport;
use quick_oxibooks::{Environment, QBContext};
use quickbooks_types::reports::types::ProfitAndLoss;
use quickbooks_types::reports::Report;
use ureq::Agent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage of the QBReport trait
    let client = Agent::new_with_defaults();

    let args = std::env::args().collect::<Vec<_>>();

    let qb_context = QBContext::new(
        Environment::PRODUCTION,
        args.get(1)
            .cloned()
            .ok_or_else(|| "Missing company ID".to_string())?,
        args.get(2)
            .cloned()
            .ok_or_else(|| "Missing access token".to_string())?,
        &client,
    )?;

    // Fetch Profit and Loss report
    match Report::get(&qb_context, &client, &ProfitAndLoss, None) {
        Ok(report) => println!(
            "Successfully fetched report: {}",
            serde_json::to_string_pretty(&report)?
        ),
        Err(e) => eprintln!("Error fetching report: {e}"),
    }

    Ok(())
}
