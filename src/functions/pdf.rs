use quickbooks_types::{QBItem, QBPDFable};
use reqwest::{Client, Method};
use tokio::io::AsyncWriteExt;

use crate::{error::APIError, Environment, QBContext};

pub async fn qb_get_pdf_bytes<T: QBItem + QBPDFable>(
    item: &T,
    qb: &QBContext,
    client: &Client,
) -> Result<Vec<u8>, APIError> {
    let Some(id) = item.id() else {
        return Err(APIError::NoIdOnGetPDF);
    };

    let request = crate::client::build_request(
        Method::GET,
        &format!("company/{}/{}/{}/pdf", qb.company_id, T::qb_id(), id),
        None::<()>,
        None,
        "application/json",
        qb.environment,
        client,
        &qb.access_token,
    )?;

    let permit = qb
        .qbo_limiter
        .acquire()
        .await
        .expect("Semaphore should not be closed");
    let response = client.execute(request).await?;
    drop(permit);

    if !response.status().is_success() {
        return Err(APIError::BadRequest(response.json().await?));
    }

    log::info!(
        "Successfully got PDF of {} with ID : {}",
        T::name(),
        item.id().ok_or(APIError::NoIdOnGetPDF)?
    );

    Ok(response.bytes().await?.into())
}

pub async fn qb_save_pdf_to_file<T: QBItem + QBPDFable>(
    item: &T,
    file_name: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<(), APIError> {
    let bytes = qb_get_pdf_bytes(item, qb, client).await?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_name)
        .await?;
    let amt = file.write(&bytes).await?;

    if bytes.len() != amt {
        log::error!("Couldn't write all the bytes of file : {}", file_name);
        return Err(APIError::ByteLengthMismatch);
    }

    log::info!(
        "Successfully saved PDF of {} #{} to {}",
        T::name(),
        item.id().ok_or(APIError::NoIdOnGetPDF)?,
        file_name
    );
    Ok(())
}
