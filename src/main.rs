mod quickbook;
use quickbook::QuickBooks;

#[tokio::main]
async fn main() {
    
    let qb = QuickBooks::new_from_env("4620816365257778210").await;
    
    let ci = qb.company_info("4620816365257778210").await.unwrap();
    println!("{:?}", serde_json::to_string_pretty(&ci).unwrap());

    let inv = qb.get_invoice_by_doc_num("1010").await.unwrap();
    println!("{:?}", serde_json::to_string_pretty(&inv).unwrap());
}
