use quickbooks_types::{QBCreatable, QBDeletable, QBItem, QBSendable};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};

use crate::{client::QBContext, error::APIError};

/// Sends a request to the QuickBooks API endpoint with the given parameters
///
/// # Arguments
///
/// * `qb` - The context containing authentication details
/// * `method` - The HTTP method for the request
/// * `path` - The path for the API request URL
/// * `body` - Optional request body to send
/// * `content_type` - Optional content type header value
/// * `query` - Optional query parameters
pub(crate) async fn qb_request<T, U>(
    qb: &QBContext,
    client: &Client,
    method: Method,
    path: &str,
    body: Option<T>,
    content_type: Option<&str>,
    query: Option<&[(&str, &str)]>,
) -> Result<U, APIError>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    let request = crate::client::build_request(
        method,
        path,
        body,
        query,
        content_type.unwrap_or("application/json"),
        qb.environment,
        &client,
        &qb.access_token,
    )?;
    let response = client.execute(request).await?;
    if !response.status().is_success() {
        return Err(APIError::BadRequest(response.text().await?));
    }
    Ok(response.json().await?)
}

/// Internal struct that Quickbooks returns most
/// of the time when interacting with the API
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct QBResponse<T> {
    #[serde(
        alias = "Item",
        alias = "Account",
        alias = "Attachabe",
        alias = "Invoice",
        alias = "Attachable",
        alias = "Bill",
        alias = "CompanyInfo",
        alias = "Customer",
        alias = "Employee",
        alias = "Estimate",
        alias = "Payment",
        alias = "SalesReceipt",
        alias = "Vendor"
    )]
    object: T,
    time: String,
}

/// Creates the given item using the context given, but first
/// checks if the item is suitable to be created.
pub async fn qb_create<T: QBItem + QBCreatable>(
    item: &T,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError> {
    if !item.can_create() {
        return Err(APIError::CreateMissingItems);
    }

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::POST,
        &format!("company/{}/{}", qb.company_id, T::qb_id()),
        Some(item),
        None,
        None,
    )
    .await?;

    log::info!(
        "Successfully created {} with ID of {}",
        T::name(),
        response
            .object
            .id()
            .unwrap_or(&"No ID on QB object after creation".into())
    );

    Ok(response.object)
}

/// Deletes the given item using the ID
/// returns an error if the item has no ID and sync token
/// available or if the request itself fails
pub async fn qb_delete<T: QBItem + QBDeletable>(
    item: &T,
    qb: &QBContext,
    client: &Client,
) -> Result<QBDeleted, APIError> {
    let (Some(_), Some(id)) = (item.sync_token(), item.id()) else {
        return Err(APIError::DeleteMissingItems);
    };

    let delete_object: QBToDelete = item.into();

    let response: QBResponse<QBDeleted> = qb_request(
        qb,
        client,
        Method::POST,
        &format!("company/{}/{}?operation=delete", qb.company_id, T::qb_id()),
        Some(delete_object),
        None,
        None,
    )
    .await?;

    log::info!("Successfully deleted {} with ID of {}", T::name(), id);

    Ok(response.object)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct QBToDelete {
    id: String,
    sync_token: String,
}

// ! For some reason TryFrom won't compile, however it is always checked if there is an ID and SyncToken before using this atm
impl<T: QBItem> From<&T> for QBToDelete {
    fn from(value: &T) -> Self {
        match (value.id().cloned(), value.sync_token().cloned()) {
            (Some(id), Some(sync_token)) => Self { id, sync_token },
            (_, _) => panic!("Couldnt delete QBItem, no ID or SyncToken available!"), // TODO Make this not possible
        }
    }
}

/// Information about the deleted object from `qb_delete`
#[derive(Deserialize, Debug, Default)]
pub struct QBDeleted {
    pub status: String,
    pub domain: String,
    #[serde(rename = "Id")]
    pub id: String,
}

/// Query the quickbooks context using the query string,
/// The type determines what type of quickbooks object you are
/// Query QuickBooks for objects matching the query string
///
/// Builds a query using the `query_str` and queries for objects of
/// type `T`. Returns up to `max_results` objects in a `Vec`.
///
/// The `query_str` parameter will be placed into the query
/// like so:
/// ```ignore
///  "select * from {type_name} {query_str} MAXRESULTS {max_results}"
/// ```
pub async fn qb_query<T: QBItem>(
    query_str: &str,
    max_results: usize,
    qb: &QBContext,
    client: &Client,
) -> Result<Vec<T>, APIError> {
    let response: QueryResponseExt<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/query", qb.company_id),
        None::<()>,
        None,
        Some(&[(
            "query",
            &format!(
                "select * from {} {query_str} MAXRESULTS {max_results}",
                T::name()
            ),
        )]),
    )
    .await?;

    if response.query_response.items.is_empty() {
        log::warn!("Queried no items for query : {query_str}");
        Err(APIError::NoQueryObjects(query_str.into()))
    } else {
        log::info!(
            "Successfully Queried {} {}(s) for query string : {query_str}",
            response.query_response.items.len(),
            T::name()
        );
        Ok(response.query_response.items)
    }
}

/// Gets a single object by ID from the QuickBooks API
///
/// Handles retrieving a QBItem via query,
/// refer to `qb_query` for more details
pub async fn qb_query_single<T: QBItem>(
    query_str: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError> {
    Ok(qb_query(query_str, 1, qb, client).await?.swap_remove(0))
}

/// Internal struct that Quickbooks returns when querying objects
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
struct QueryResponse<T> {
    total_count: i64,
    #[serde(
        alias = "Item",
        alias = "Account",
        alias = "Invoice",
        alias = "Attachable",
        alias = "Bill",
        alias = "CompanyInfo",
        alias = "Customer",
        alias = "Employee",
        alias = "Estimate",
        alias = "Payment",
        alias = "SalesReceipt",
        alias = "Vendor"
    )]
    items: Vec<T>,
    start_position: i64,
    max_results: i64,
}

/// Internal struct that Quickbooks returns when querying objects
#[derive(Debug, Clone, Deserialize)]
struct QueryResponseExt<T> {
    #[serde(default, rename = "QueryResponse")]
    query_response: QueryResponse<T>,
    #[allow(dead_code)]
    time: String,
}

/// Read the object by ID from quickbooks context
/// and write it to an item
pub async fn qb_read<T: QBItem>(
    item: &mut T,
    qb: &QBContext,
    client: &Client,
) -> Result<(), APIError> {
    let Some(id) = item.id() else {
        return Err(APIError::NoIdOnRead);
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<()>,
        None,
        None,
    )
    .await?;

    log::info!(
        "Successfully Read {} object with ID : {}",
        T::name(),
        response
            .object
            .id()
            .unwrap_or(&"No ID after reading QB Object".into())
    );

    *item = response.object;

    Ok(())
}

/// Retrieves an object by ID from quickbooks context
pub async fn qb_get_single<T: QBItem>(
    id: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError> {
    let response: QBResponse<T> = qb_request(
        qb,
        client,
        Method::GET,
        &format!("company/{}/{}/{}", qb.company_id, T::qb_id(), id),
        None::<()>,
        None,
        None,
    )
    .await?;
    Ok(response.object)
}

/// Send email of the object to the email given through quickbooks context
pub async fn qb_send_email<T: QBItem + QBSendable>(
    item: &T,
    email: &str,
    qb: &QBContext,
    client: &Client,
) -> Result<T, APIError> {
    let Some(id) = item.id() else {
        return Err(APIError::NoIdOnSend);
    };

    let response: QBResponse<T> = qb_request(
        qb,
        client,
        reqwest::Method::POST,
        &format!("company/{}/{}/{}/send", qb.company_id, T::qb_id(), id),
        None::<()>,
        None,
        Some(&[("sendTo", email)]),
    )
    .await?;
    log::info!("Successfully Sent {} object with ID : {}", T::name(), id);
    Ok(response.object)
}

#[cfg(feature = "attachments")]
pub mod attachment {
    use std::path::{Path, PathBuf};

    use base64::Engine;
    use quickbooks_types::{content_type_from_ext, Attachable, QBAttachable};
    use reqwest::{
        header::{self, HeaderValue}, multipart::{Form, Part}, Client, Method, Request
    };

    use crate::{error::APIError, QBContext};

    async fn _make_file_part(file_name: impl AsRef<Path>) -> Result<Part, APIError> {
        let buf = tokio::fs::read(&file_name).await?;
        let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(buf);

        let file_headers = {
            let mut headers = header::HeaderMap::new();
            headers.append(
                "Content-Transfer-Encoding",
                HeaderValue::from_static("base64"),
            );
            headers
        };

        // Would've returned an error already if it was directory, safe to unwrap
        let ext: PathBuf = file_name.as_ref().to_path_buf();
        let ct = content_type_from_ext(ext.extension().unwrap().to_str().unwrap());

        let file_part = Part::bytes(encoded.into_bytes())
            .mime_str(ct)?
            .file_name(
                file_name
                    .as_ref()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            )
            .headers(file_headers);

        Ok(file_part)
    }

    /// Attach a file to another Quickbooks objct
    /// via a `Attachable` object
    ///
    /// Uploads the file and makes the `attachable` object
    /// in QuickBooks.
    pub async fn qb_upload(
        attachable: &Attachable,
        qb: &QBContext,
        client: &Client,
    ) -> Result<Attachable, APIError> {
        if !attachable.can_upload() {
            return Err(APIError::AttachableUploadMissingItems);
        }

        let request = make_upload_request(attachable, qb, client).await?;

        let response = client.execute(request).await?;

        if !response.status().is_success() {
            return Err(APIError::BadRequest(response.text().await?));
        }

        let mut qb_response: AttachableResponseExt = response.json().await?;
        if qb_response.ar.is_empty() {
            return Err(APIError::NoAttachableObjects);
        };

        let obj = qb_response.ar.swap_remove(0).attachable;

        log::info!("Sent attachment : {:?}", obj.file_name.as_ref().unwrap());

        Ok(obj)
    }

    async fn make_upload_request(
        attachable: &Attachable,
        qb: &QBContext,
        client: &Client,
    ) -> Result<Request, APIError> {
        let file_name = attachable
            .file_name
            .as_ref()
            .ok_or(APIError::AttachableUploadMissingItems)?;

        let path = format!("company/{}/upload", qb.company_id);
        let url = crate::client::build_url(qb.environment, &path, Some(&[]))?;
        let request_headers = crate::client::build_headers("application/pdf", &qb.access_token)?;

        let json_body = serde_json::to_string(attachable).expect("Couldn't Serialize Attachment");
        let json_part = Part::text(json_body).mime_str("application/json")?;

        let file_part = _make_file_part(file_name).await?;

        let multipart = Form::new()
            .part("file_metadata_01", json_part)
            .part("file_content_01", file_part);

        Ok(client
            .request(Method::POST, url)
            .headers(request_headers)
            .multipart(multipart)
            .build()?)
    }

    #[derive(Debug, serde::Deserialize)]
    struct AttachableResponseExt {
        #[serde(rename = "AttachableResponse")]
        ar: Vec<AttachableResponse>,
        #[allow(dead_code)]
        time: String,
    }

    #[derive(serde::Deserialize, Debug)]
    struct AttachableResponse {
        #[serde(rename = "Attachable")]
        attachable: Attachable,
    }
}

#[cfg(feature = "pdf")]
pub mod pdf {
    use quickbooks_types::{QBItem, QBPDFable};
    use reqwest::{Client, Method};
    use tokio::io::AsyncWriteExt;

    use crate::{error::APIError, Environment};

    pub async fn qb_get_pdf_bytes<T: QBItem + QBPDFable>(
        item: &T,
        client: &Client,
        environment: Environment,
        company_id: &str,
        access_token: &str,
    ) -> Result<Vec<u8>, APIError> {
        let Some(id) = item.id() else {
            return Err(APIError::NoIdOnGetPDF);
        };

        let request = crate::client::build_request(
            Method::GET,
            &format!("company/{}/{}/{}/pdf", company_id, T::qb_id(), id),
            None::<()>,
            None,
            "application/json",
            environment,
            client,
            access_token,
        )?;

        let response = client.execute(request).await?;

        if !response.status().is_success() {
            return Err(APIError::BadRequest(response.text().await?));
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
        client: &Client,
        environment: Environment,
        company_id: &str,
        access_token: &str,
    ) -> Result<(), APIError> {
        let bytes = qb_get_pdf_bytes(item, client, environment, company_id, access_token).await?;
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
}
